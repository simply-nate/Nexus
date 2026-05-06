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
    ShapeMismatch(Vec<usize>, Vec<usize>),
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
            NexusError::ShapeMismatch(s1, s2) => write!(f, "Shape mismatch: {:?} vs {:?}", s1, s2),
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
    Max = 5,
    Min = 6,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Adverb {
    Reduce = 1,
    Scan = 2,
    Each = 3,
    Table = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedTensor {
    pub data: Vec<f64>,
    pub shape: Vec<usize>,
    pub ontic_type: u64,
}

impl TypedTensor {
    pub fn is_scalar(&self) -> bool {
        self.shape.is_empty() || (self.shape.len() == 1 && self.shape[0] == 1)
    }

    pub fn scalar_value(&self) -> f64 {
        if self.data.is_empty() { 0.0 } else { self.data[0] }
    }
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
    Contradiction(Signature),
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

    pub fn define(&mut self, alias: &str) -> Result<u64, NexusError> {
        if self.aliases.contains_key(alias) {
            return Err(NexusError::TypeAlreadyExists(alias.to_string()));
        }
        let id = self.next_auto_id;
        self.next_auto_id += 1;
        self.aliases.insert(alias.to_string(), id);
        Ok(id)
    }

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
// Math Execution Engine
// ==========================================
fn exec_dyadic(op: Op, a: f64, b: f64) -> Result<f64, NexusError> {
    match op {
        Op::Add => Ok(a + b),
        Op::Subtract => Ok(a - b),
        Op::Multiply => Ok(a * b),
        Op::Divide => {
            if b == 0.0 {
                return Err(NexusError::ExecutionError("Division by zero".to_string()));
            }
            Ok(a / b)
        }
        Op::Max => Ok(a.max(b)),
        Op::Min => Ok(a.min(b)),
    }
}

// ==========================================
// Nexus Context
// ==========================================
pub type SavepointId = usize;

pub struct NexusContext {
    pub agent_id: u64,
    registry: TypeRegistry,
    ledger: ConsistencyLedger,
    stack: Vec<TypedTensor>,
    savepoints: Vec<(Vec<TypedTensor>, ConsistencyLedger)>,
    goal_text: Option<String>,
}

impl NexusContext {
    pub fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            registry: TypeRegistry::new(),
            ledger: ConsistencyLedger::new(),
            stack: Vec::new(),
            savepoints: Vec::new(),
            goal_text: None,
        }
    }

    pub fn registry(&self) -> &TypeRegistry { &self.registry }
    pub fn registry_mut(&mut self) -> &mut TypeRegistry { &mut self.registry }
    pub fn ledger(&mut self) -> &mut ConsistencyLedger { &mut self.ledger }

    // ---- Stack: Core ----

    pub fn push_tensor(&mut self, data: Vec<f64>, shape: Vec<usize>, ontic_type: u64) {
        self.stack.push(TypedTensor { data, shape, ontic_type });
    }

    pub fn push_scalar(&mut self, value: f64, ontic_type: u64) {
        self.push_tensor(vec![value], vec![], ontic_type);
    }

    pub fn pop(&mut self) -> Result<TypedTensor, NexusError> {
        self.stack.pop().ok_or(NexusError::StackUnderflow)
    }

    /// Look at the top of the stack without consuming it.
    pub fn peek(&self) -> Result<&TypedTensor, NexusError> {
        self.stack.last().ok_or(NexusError::StackUnderflow)
    }

    /// Current number of items on the stack.
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    // ---- Stack: Manipulation via Pick ----

    /// Unified stack manipulation. Replaces dup, swap, over, rot, drop.
    ///
    /// Consumes `max(indices) + 1` items from the top of the stack,
    /// then pushes them back in the order specified by `indices`.
    ///
    /// Examples:
    /// - `pick(&[0, 0])`    → dup   (consume 1, push twice)
    /// - `pick(&[1, 0])`    → swap  (consume 2, reverse)
    /// - `pick(&[0, 1, 0])` → over  (consume 2, push [top, second, top])
    /// - `pick(&[2, 0, 1])` → rot   (consume 3, rotate)
    pub fn pick(&mut self, _indices: &[usize]) -> Result<(), NexusError> {
        todo!("pick: unified stack manipulation not yet implemented")
    }

    /// Discard the top item on the stack.
    pub fn drop_top(&mut self) -> Result<(), NexusError> {
        todo!("drop_top: not yet implemented")
    }

    // ---- Operations ----

    pub fn apply(&mut self, op: Op, expected_inputs: &[u64], outputs: &[u64]) -> Result<LedgerVerdict, NexusError> {
        if self.stack.len() < expected_inputs.len() {
            return Err(NexusError::StackUnderflow);
        }

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

        // Perform basic array broadcasting for 2-input operations
        if actual_inputs.len() == 2 && outputs.len() == 1 {
            let t_a = &actual_inputs[0];
            let t_b = &actual_inputs[1];
            
            let (data, shape) = if t_a.is_scalar() && t_b.is_scalar() {
                (vec![exec_dyadic(op, t_a.scalar_value(), t_b.scalar_value())?], vec![])
            } else if t_a.is_scalar() {
                let s = t_a.scalar_value();
                let d = t_b.data.iter().map(|&v| exec_dyadic(op, s, v)).collect::<Result<Vec<_>, _>>()?;
                (d, t_b.shape.clone())
            } else if t_b.is_scalar() {
                let s = t_b.scalar_value();
                let d = t_a.data.iter().map(|&v| exec_dyadic(op, v, s)).collect::<Result<Vec<_>, _>>()?;
                (d, t_a.shape.clone())
            } else {
                // Same shape assumed for v0.2
                if t_a.shape != t_b.shape {
                    return Err(NexusError::ShapeMismatch(t_a.shape.clone(), t_b.shape.clone()));
                }
                let mut d = Vec::with_capacity(t_a.data.len());
                for (a, b) in t_a.data.iter().zip(t_b.data.iter()) {
                    d.push(exec_dyadic(op, *a, *b)?);
                }
                (d, t_a.shape.clone())
            };
            
            self.push_tensor(data, shape, outputs[0]);
        } else {
            // Unhandled arithmetic fallback
            for &out_type in outputs {
                self.push_scalar(0.0, out_type);
            }
        }

        let verdict = self.ledger.check(op, expected_inputs, outputs);
        Ok(verdict)
    }

    pub fn apply_adverb(&mut self, adverb: Adverb, op: Op, expected_inputs: &[u64], outputs: &[u64]) -> Result<LedgerVerdict, NexusError> {
        // Only implemented Reduce for 1D arrays for now
        if adverb == Adverb::Reduce {
            if self.stack.is_empty() { return Err(NexusError::StackUnderflow); }
            let t = self.pop()?;
            if t.ontic_type != expected_inputs[0] {
                return Err(NexusError::ExecutionError(format!("Type mismatch")));
            }
            if t.data.is_empty() {
                return Err(NexusError::ExecutionError("Reduce on empty array".to_string()));
            }

            let mut acc = t.data[0];
            for &v in t.data.iter().skip(1) {
                acc = exec_dyadic(op, acc, v)?;
            }
            
            self.push_scalar(acc, outputs[0]);
            let verdict = self.ledger.check(op, expected_inputs, outputs); // Ledger tracks underlying verb context
            Ok(verdict)
        } else {
            Err(NexusError::ExecutionError(format!("{:?} not fully implemented", adverb)))
        }
    }

    // ---- Bridges / Conversion ----

    pub fn convert_to(&mut self, target_type: u64) -> Result<(), NexusError> {
        let t = self.pop()?;
        if t.ontic_type == target_type {
            self.push_tensor(t.data, t.shape, t.ontic_type);
            return Ok(());
        }

        let bridge = self.registry.get_bridge(t.ontic_type, target_type);
        if let Some((op, factor)) = bridge {
            let mut new_data = Vec::with_capacity(t.data.len());
            for &v in &t.data {
                new_data.push(exec_dyadic(op, v, factor)?);
            }
            self.push_tensor(new_data, t.shape, target_type);
            Ok(())
        } else {
            self.push_tensor(t.data, t.shape, t.ontic_type);
            Err(NexusError::ConversionNotFound(t.ontic_type, target_type))
        }
    }

    // ---- Serialization ----

    /// Serialize the entire context (registry, ledger, stack) to JSON.
    pub fn serialize(&self) -> Result<String, NexusError> {
        todo!("serialize: context serialization not yet implemented")
    }

    /// Restore a context from a JSON string.
    pub fn deserialize(_json: &str) -> Result<NexusContext, NexusError> {
        todo!("deserialize: context deserialization not yet implemented")
    }

    // ---- Assert-Gated Effects ----

    /// Check that the ledger has no contradictions since the last savepoint.
    /// On pass: creates a new savepoint. On fail: rolls back to the last savepoint.
    pub fn assert_consistent(&mut self) -> Result<(), NexusError> {
        todo!("assert_consistent: not yet implemented")
    }

    /// Assert the top of the stack has the expected ontic type.
    /// On pass: creates a new savepoint. On fail: rolls back.
    pub fn assert_type(&mut self, _expected: u64) -> Result<(), NexusError> {
        todo!("assert_type: not yet implemented")
    }

    /// Assert the top of the stack has the expected shape.
    /// On pass: creates a new savepoint. On fail: rolls back.
    pub fn assert_shape(&mut self, _expected: &[usize]) -> Result<(), NexusError> {
        todo!("assert_shape: not yet implemented")
    }

    /// Create a manual savepoint. Returns an ID for later rollback.
    pub fn savepoint(&mut self) -> SavepointId {
        todo!("savepoint: not yet implemented")
    }

    /// Roll back to a previous savepoint, restoring stack and ledger state.
    pub fn rollback(&mut self, _to: SavepointId) {
        todo!("rollback: not yet implemented")
    }

    // ---- Goals ----

    /// Set a goal description. Echoed back in REPL responses and persisted in serialization.
    pub fn goal(&mut self, description: &str) {
        self.goal_text = Some(description.to_string());
    }

    /// Clear the current goal.
    pub fn goal_done(&mut self) {
        self.goal_text = None;
    }

    /// Get the current goal, if set.
    pub fn current_goal(&self) -> Option<&str> {
        self.goal_text.as_deref()
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
pub extern "C" fn nexus_push_tensor(
    ctx: *mut NexusContext, 
    data_ptr: *const f64, data_len: usize,
    shape_ptr: *const usize, shape_len: usize,
    ontic_type: u64
) -> i32 {
    if ctx.is_null() { return -1; }
    let context = unsafe { &mut *ctx };
    
    let data = if data_ptr.is_null() || data_len == 0 { vec![] } else { unsafe { std::slice::from_raw_parts(data_ptr, data_len).to_vec() } };
    let shape = if shape_ptr.is_null() || shape_len == 0 { vec![] } else { unsafe { std::slice::from_raw_parts(shape_ptr, shape_len).to_vec() } };
    
    context.push_tensor(data, shape, ontic_type);
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn nexus_apply(
    ctx: *mut NexusContext,
    op: u32,
    input_types: *const u64, input_len: usize,
    output_types: *const u64, output_len: usize,
) -> i32 {
    if ctx.is_null() { return -1; }
    let context = unsafe { &mut *ctx };
    let op = match op {
        1 => Op::Add, 2 => Op::Subtract, 3 => Op::Multiply, 4 => Op::Divide, 5 => Op::Max, 6 => Op::Min, _ => return -2,
    };

    let inputs = if input_types.is_null() || input_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(input_types, input_len) } };
    let outputs = if output_types.is_null() || output_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(output_types, output_len) } };

    match context.apply(op, inputs, outputs) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nexus_apply_adverb(
    ctx: *mut NexusContext,
    adverb: u32, op: u32,
    input_types: *const u64, input_len: usize,
    output_types: *const u64, output_len: usize,
) -> i32 {
    if ctx.is_null() { return -1; }
    let context = unsafe { &mut *ctx };
    
    let adverb = match adverb { 1 => Adverb::Reduce, 2 => Adverb::Scan, 3 => Adverb::Each, 4 => Adverb::Table, _ => return -2 };
    let op = match op { 1 => Op::Add, 2 => Op::Subtract, 3 => Op::Multiply, 4 => Op::Divide, 5 => Op::Max, 6 => Op::Min, _ => return -2 };

    let inputs = if input_types.is_null() || input_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(input_types, input_len) } };
    let outputs = if output_types.is_null() || output_len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(output_types, output_len) } };

    match context.apply_adverb(adverb, op, inputs, outputs) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nexus_context_free(ctx: *mut NexusContext) {
    if !ctx.is_null() {
        unsafe { let _ = Box::from_raw(ctx); }
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
        ctx.push_scalar(42.0, SCALAR);
        let val = ctx.pop().unwrap();
        assert_eq!(val.data, vec![42.0]);
        assert_eq!(val.shape, Vec::<usize>::new());
        assert_eq!(val.ontic_type, SCALAR);
    }

    #[test]
    fn test_tensor_math() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;

        ctx.push_tensor(vec![2.0, 3.0, 4.0], vec![3], METER);
        ctx.push_tensor(vec![5.0, 5.0, 5.0], vec![3], METER);

        ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();

        let res = ctx.pop().unwrap();
        assert_eq!(res.data, vec![10.0, 15.0, 20.0]);
        assert_eq!(res.shape, vec![3]);
        assert_eq!(res.ontic_type, area);
    }

    #[test]
    fn test_scalar_broadcast() {
        let mut ctx = NexusContext::new(1);
        let double_meters = 0x1020;
        
        ctx.push_tensor(vec![1.0, 2.0, 3.0], vec![3], METER);
        ctx.push_scalar(2.0, SCALAR);
        
        ctx.apply(Op::Multiply, &[METER, SCALAR], &[double_meters]).unwrap();
        
        let res = ctx.pop().unwrap();
        assert_eq!(res.data, vec![2.0, 4.0, 6.0]);
        assert_eq!(res.ontic_type, double_meters);
    }
    
    #[test]
    fn test_adverb_reduce() {
        let mut ctx = NexusContext::new(1);
        let total = 0x1030;
        
        ctx.push_tensor(vec![1.0, 2.0, 3.0, 4.0], vec![4], METER);
        ctx.apply_adverb(Adverb::Reduce, Op::Add, &[METER], &[total]).unwrap();
        
        let res = ctx.pop().unwrap();
        assert_eq!(res.data, vec![10.0]);
        assert_eq!(res.shape, Vec::<usize>::new());
        assert_eq!(res.ontic_type, total);
    }
}
