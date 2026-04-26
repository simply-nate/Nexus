use std::collections::HashMap;
use std::fmt;

// ==========================================
// Layer 0 Axiomatic Types
// ==========================================
pub const NULL_TYPE: u64 = 0x0000;
pub const SCALAR: u64    = 0x0001;
pub const METER: u64     = 0x0002;
pub const KILOGRAM: u64  = 0x0003;
pub const SECOND: u64    = 0x0004;
pub const BIT: u64       = 0x0005;

// ==========================================
// Errors
// ==========================================
#[derive(Debug, Clone)]
pub enum NexusError {
    TypeAlreadyExists(String),
    TypeNotFound(String),
    StackUnderflow,
    ConversionNotFound(u64, u64),
    ExecutionError(String),
}

impl std::error::Error for NexusError {}

impl fmt::Display for NexusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NexusError::TypeAlreadyExists(name) => write!(f, "Type alias already exists: {}", name),
            NexusError::TypeNotFound(name) => write!(f, "Type alias not found: {}", name),
            NexusError::StackUnderflow => write!(f, "Not enough values on stack"),
            NexusError::ConversionNotFound(from, to) => write!(f, "No bridge found to convert from {} to {}", from, to),
            NexusError::ExecutionError(e) => write!(f, "Execution error: {}", e),
        }
    }
}

// ==========================================
// Core Enums and Structs
// ==========================================
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    Add = 1,
    Subtract = 2,
    Multiply = 3,
    Divide = 4,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypedValue {
    pub value: f64,
    pub ontic_type: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub op: Op,
    pub inputs: Vec<u64>,
    pub outputs: Vec<u64>,
}

pub enum LedgerVerdict {
    Novel,
    Consistent,
    Contradiction(Signature), // The prior signature that contradicts
}

// ==========================================
// Type Registry
// ==========================================
#[derive(Debug, Clone)]
pub struct TypeRegistry {
    aliases: HashMap<String, u64>,
    bridges: HashMap<(u64, u64), (Op, f64)>,
    next_auto_id: u64,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut aliases = HashMap::new();
        aliases.insert("NULL_TYPE".to_string(), NULL_TYPE);
        aliases.insert("SCALAR".to_string(), SCALAR);
        aliases.insert("METER".to_string(), METER);
        aliases.insert("KILOGRAM".to_string(), KILOGRAM);
        aliases.insert("SECOND".to_string(), SECOND);
        aliases.insert("BIT".to_string(), BIT);

        Self {
            aliases,
            bridges: HashMap::new(),
            next_auto_id: 0x1000,
        }
    }

    /// Automatically assigns the next available u64 ID to the alias
    pub fn define(&mut self, alias: &str) -> Result<u64, NexusError> {
        if self.aliases.contains_key(alias) {
            return Err(NexusError::TypeAlreadyExists(alias.to_string()));
        }
        let id = self.next_auto_id;
        self.next_auto_id += 1;
        self.aliases.insert(alias.to_string(), id);
        Ok(id)
    }

    /// Explicitly assigns a specific u64 ID to the alias (useful for global constants)
    pub fn define_explicit(&mut self, alias: &str, id: u64) -> Result<(), NexusError> {
        if self.aliases.contains_key(alias) {
            return Err(NexusError::TypeAlreadyExists(alias.to_string()));
        }
        if id >= self.next_auto_id {
            self.next_auto_id = id + 1;
        }
        self.aliases.insert(alias.to_string(), id);
        Ok(())
    }

    pub fn get(&self, alias: &str) -> Result<u64, NexusError> {
        self.aliases.get(alias).copied().ok_or_else(|| NexusError::TypeNotFound(alias.to_string()))
    }

    pub fn bridge(&mut self, from: u64, to: u64, op: Op, factor: f64) {
        self.bridges.insert((from, to), (op, factor));
    }

    pub fn get_bridge(&self, from: u64, to: u64) -> Option<(Op, f64)> {
        self.bridges.get(&(from, to)).copied()
    }
}

// ==========================================
// Consistency Ledger
// ==========================================
#[derive(Debug, Clone, Default)]
pub struct ConsistencyLedger {
    history: HashMap<(Op, Vec<u64>), Vec<u64>>,
}

impl ConsistencyLedger {
    pub fn new() -> Self {
        Self { history: HashMap::new() }
    }

    pub fn check(&mut self, op: Op, inputs: &[u64], outputs: &[u64]) -> LedgerVerdict {
        let key = (op, inputs.to_vec());
        if let Some(prior_outputs) = self.history.get(&key) {
            if prior_outputs == outputs {
                LedgerVerdict::Consistent
            } else {
                LedgerVerdict::Contradiction(Signature {
                    op,
                    inputs: inputs.to_vec(),
                    outputs: prior_outputs.clone(),
                })
            }
        } else {
            self.history.insert(key, outputs.to_vec());
            LedgerVerdict::Novel
        }
    }
}

// ==========================================
// Nexus Context
// ==========================================
pub struct NexusContext {
    pub agent_id: u64,
    registry: TypeRegistry,
    ledger: ConsistencyLedger,
    stack: Vec<TypedValue>,
}

impl NexusContext {
    pub fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            registry: TypeRegistry::new(),
            ledger: ConsistencyLedger::new(),
            stack: Vec::new(),
        }
    }

    pub fn registry(&self) -> &TypeRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut TypeRegistry {
        &mut self.registry
    }

    pub fn ledger(&mut self) -> &mut ConsistencyLedger {
        &mut self.ledger
    }

    pub fn push(&mut self, value: f64, ontic_type: u64) {
        self.stack.push(TypedValue { value, ontic_type });
    }

    pub fn pop(&mut self) -> Result<TypedValue, NexusError> {
        self.stack.pop().ok_or(NexusError::StackUnderflow)
    }

    pub fn apply(&mut self, op: Op, expected_inputs: &[u64], outputs: &[u64]) -> Result<LedgerVerdict, NexusError> {
        if self.stack.len() < expected_inputs.len() {
            return Err(NexusError::StackUnderflow);
        }

        // Pop inputs from stack
        let mut actual_inputs = Vec::with_capacity(expected_inputs.len());
        for expected_type in expected_inputs.iter().rev() {
            let val = self.pop()?;
            if val.ontic_type != *expected_type {
                return Err(NexusError::ExecutionError(format!(
                    "Type mismatch. Expected {}, found {}", expected_type, val.ontic_type
                )));
            }
            actual_inputs.push(val);
        }
        actual_inputs.reverse();

        // Perform basic arithmetic
        if actual_inputs.len() == 2 && outputs.len() == 1 {
            let a = actual_inputs[0].value;
            let b = actual_inputs[1].value;
            let res = match op {
                Op::Add => a + b,
                Op::Subtract => a - b,
                Op::Multiply => a * b,
                Op::Divide => {
                    if b == 0.0 {
                        return Err(NexusError::ExecutionError("Division by zero".to_string()));
                    }
                    a / b
                }
            };
            self.push(res, outputs[0]);
        } else {
            // Unhandled arithmetic sizing fallback
            for &out_type in outputs {
                self.push(0.0, out_type);
            }
        }

        // Record in ledger
        let verdict = self.ledger.check(op, expected_inputs, outputs);
        Ok(verdict)
    }

    pub fn convert_to(&mut self, target_type: u64) -> Result<(), NexusError> {
        let val = self.pop()?;
        if val.ontic_type == target_type {
            self.push(val.value, val.ontic_type);
            return Ok(());
        }

        let bridge = self.registry.get_bridge(val.ontic_type, target_type);
        if let Some((op, factor)) = bridge {
            let new_val = match op {
                Op::Add => val.value + factor,
                Op::Subtract => val.value - factor,
                Op::Multiply => val.value * factor,
                Op::Divide => val.value / factor,
            };
            self.push(new_val, target_type);
            Ok(())
        } else {
            self.push(val.value, val.ontic_type);
            Err(NexusError::ConversionNotFound(val.ontic_type, target_type))
        }
    }
}

// ==========================================
// C-FFI Surface
// ==========================================
#[unsafe(no_mangle)]
pub extern "C" fn nexus_context_new(agent_id: u64) -> *mut NexusContext {
    Box::into_raw(Box::new(NexusContext::new(agent_id)))
}

#[unsafe(no_mangle)]
pub extern "C" fn nexus_push_f64(ctx: *mut NexusContext, value: f64, ontic_type: u64) -> i32 {
    if ctx.is_null() {
        return -1;
    }
    let context = unsafe { &mut *ctx };
    context.push(value, ontic_type);
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn nexus_apply(
    ctx: *mut NexusContext,
    op: u32,
    input_types: *const u64,
    input_len: usize,
    output_types: *const u64,
    output_len: usize,
) -> i32 {
    if ctx.is_null() {
        return -1;
    }
    let context = unsafe { &mut *ctx };
    let op = match op {
        1 => Op::Add,
        2 => Op::Subtract,
        3 => Op::Multiply,
        4 => Op::Divide,
        _ => return -2, // invalid op
    };

    let inputs = if input_types.is_null() || input_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(input_types, input_len) }
    };

    let outputs = if output_types.is_null() || output_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(output_types, output_len) }
    };

    match context.apply(op, inputs, outputs) {
        Ok(_) => 0,
        Err(_) => -3, // Execution error
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nexus_context_free(ctx: *mut NexusContext) {
    if !ctx.is_null() {
        unsafe {
            let _ = Box::from_raw(ctx);
        }
    }
}

// ==========================================
// Tests
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut ctx = NexusContext::new(1);
        ctx.push(42.0, SCALAR);
        let val = ctx.pop().unwrap();
        assert_eq!(val.value, 42.0);
        assert_eq!(val.ontic_type, SCALAR);
    }

    #[test]
    fn test_registry() {
        let mut ctx = NexusContext::new(1);
        let id1 = ctx.registry_mut().define("NEW_METERS").unwrap();
        assert_eq!(id1, 0x1000);
        let id2 = ctx.registry_mut().define("NEW_SECONDS").unwrap();
        assert_eq!(id2, 0x1001);
        
        assert!(ctx.registry_mut().define("NEW_METERS").is_err());
        
        assert!(ctx.registry_mut().define_explicit("GLOBAL_TYPE", 0x2000).is_ok());
        assert_eq!(ctx.registry().get("GLOBAL_TYPE").unwrap(), 0x2000);
        
        let id3 = ctx.registry_mut().define("NEXT_AUTO").unwrap();
        assert_eq!(id3, 0x2001); // Auto-ID correctly advanced

        assert!(ctx.registry().get("MISSING").is_err());
    }

    #[test]
    fn test_bridge() {
        let mut ctx = NexusContext::new(1);
        let minutes = ctx.registry_mut().define("MINUTES").unwrap();
        ctx.registry_mut().bridge(SECOND, minutes, Op::Divide, 60.0);

        ctx.push(120.0, SECOND);
        assert!(ctx.convert_to(minutes).is_ok());

        let val = ctx.pop().unwrap();
        assert_eq!(val.value, 2.0);
        assert_eq!(val.ontic_type, minutes);
    }

    #[test]
    fn test_ledger_and_apply() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        let birds = 0x1011;

        ctx.push(3.0, METER);
        ctx.push(4.0, METER);

        let verdict = ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        assert!(matches!(verdict, LedgerVerdict::Novel));

        let res = ctx.pop().unwrap();
        assert_eq!(res.value, 12.0);
        assert_eq!(res.ontic_type, area);

        // Contradiction test
        ctx.push(3.0, METER);
        ctx.push(4.0, METER);
        let verdict2 = ctx.apply(Op::Multiply, &[METER, METER], &[birds]).unwrap();
        assert!(matches!(verdict2, LedgerVerdict::Contradiction(_)));
    }
}
