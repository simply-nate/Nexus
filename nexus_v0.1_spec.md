# NEXUS
## Ontic Protocol
**Product Specification v0.1**

**Status:** Draft — For Implementation  
**Target:** Language-agnostic embeddable library (implemented in Rust)

---

## 1. What Is Nexus?
Nexus is a language-agnostic, embeddable library for semantic type tracking. It attaches meaning to data by associating values with typed identifiers and enforces consistency by recording how types are transformed across operations.

The core problem it solves: in any system where multiple agents (human or AI) produce and consume data, the same numeric value can mean completely different things depending on context. Nexus makes that context explicit, portable, and auditable.

**Design principle:** Nexus does not tell you what the universe is. It forces you to be self-consistent about what you observe.

### 1.1 What It Is Not
*   **Not a type system for a programming language:** It tracks semantic meaning at runtime, not static types at compile time.
*   **Not an exhaustive physics simulation engine:** It does not force you to represent $Meters^{1000000}$ if you take the geometric mean of a million lengths. Nexus tracks *nominal* semantics, not structural dimensional analysis.
*   **Not a database or query layer.**
*   **Not a standalone language:** It is a library you embed in existing systems.

### 1.2 Why a Library, Not a Language
A standalone language requires a new toolchain, new editor support, and developer buy-in before a single line of real work can be written. A library can be imported into existing Rust, Python, or JavaScript projects on day one. This constraint is intentional and non-negotiable for v0.1.

---

## 2. Core Concepts

### 2.1 Ontic Identifiers
Every semantic type in Nexus is represented as a `u64` integer constant. String labels are human-readable aliases, not the canonical identity. Integer comparison is $O(1)$ and immune to typos.

```rust
// Types are u64 constants
pub const METER: u64 = 0x0002;
pub const SECOND: u64 = 0x0004;
pub const AREA: u64 = 0x0010;  // user-defined derived type
```

### 2.2 The Type Registry
Before a type can be used, it must be registered. The registry allows declaring new aliases or referring to existing ones.

```rust
// Declare new types. Fails if the alias or ID already exists.
ctx.registry().define("METERS", 0x1001)?;

// Reference an existing type. Fails if not found — catching typos immediately.
let meters = ctx.registry().get("METERS")?;
```

This catches common bugs: accidental re-declaration of a type and typos in references.

### 2.3 The Stack & Explicit Typing
Nexus exposes a stack-based (or data-flow) API. Values are pushed onto the stack with a type, and operations consume typed inputs and produce typed outputs. All type information travels with the data.

```rust
ctx.push(3.0, meters);
ctx.push(3.0, meters);

// The operation consumes 2 METERS and outputs 1 AREA.
ctx.apply(Op::Multiply, &[meters, meters], &[area])?;
// Stack now contains: [ 9.0 :: AREA ]
```

The type arrays on each operation are declarations: "this operation expects these types as input and produces these types as output." Nexus does not infer derived types—you state them explicitly to avoid combinatorial explosion of types like $Meters^{1000000}$. 

### 2.4 The Consistency Ledger
Nexus does not validate operations against a pre-built ontology. Instead, it records what each operation claims to do and flags contradictions across uses.

1.  **First use:** `apply(Op::Multiply, &[METERS, METERS], &[AREA])` is recorded in the ledger.
2.  **Second use:** `apply(Op::Multiply, &[METERS, METERS], &[BIRDS])` triggers a contradiction warning.

This is an empirical consistency check, not a physics check. Nexus knows only that you previously claimed multiplying two `METERS` produces `AREA`, and flags the discrepancy for review.

---

## 3. Registry Architecture

### 3.1 Scoped Inheritance
The type registry has two tiers:
*   **Global (read-only):** Ships with Nexus. Contains fundamental concepts. Cannot be modified at runtime.
*   **Local (per-context):** Each `NexusContext` has its own registry. Reads check local first, then fall through to global. Writes go to local only.

### 3.2 Axiomatic Layer 0 (Built-in Constants)
The following `u64` constants are reserved and ship with the global registry:

| Constant   | Value  | Domain |
| :--- | :--- | :--- |
| `NULL_TYPE` | `0x0000` | Void / untyped |
| `SCALAR`   | `0x0001` | Pure magnitude (dimensionless) |
| `METER`    | `0x0002` | Length (SI base) |
| `KILOGRAM` | `0x0003` | Mass (SI base) |
| `SECOND`   | `0x0004` | Time (SI base) |
| `BIT`      | `0x0005` | Information |

User-defined types begin at `0x1000`. The range `0x0000–0x0FFF` is reserved for the standard library.

---

## 4. v0.1 API Specification

### 4.1 Context Lifecycle
```rust
// Create a context (owns its local registry + ledger)
let mut ctx = NexusContext::new(agent_id);
```

### 4.2 Type Registration
```rust
// Automatically assign IDs to new types (convenience)
let meters = ctx.registry_mut().define("METERS")?;
let meters_sq = ctx.registry_mut().define("METERS_SQ")?;
let seconds = ctx.registry_mut().define("SECONDS")?;
let birds = ctx.registry_mut().define("BIRDS")?;

// Or explicitly assign a type ID (useful for global constants across agents)
ctx.registry_mut().define_explicit("PESOS", 0x2001)?;

// Fetch an existing type ID safely
let pesos = ctx.registry().get("PESOS")?;
```

### 4.3 Stack Operations
```rust
// Push a value with a type
ctx.push(3.0_f64, meters);

// Apply an operation with explicit type signature
ctx.apply(
    Op::Multiply, 
    &[meters, meters], 
    &[meters_sq]
)?;

// Pop result
let result: TypedValue = ctx.pop()?;
// result.value == 9.0
// result.ontic_type == area
```

### 4.4 Ledger Query
```rust
// Check if an operation signature is consistent with history
let verdict = ctx.ledger().check(
    Op::Multiply,
    &[meters, meters],
    &[birds], // contradiction!
);

match verdict {
    LedgerVerdict::Novel => { /* first use, recorded */ }
    LedgerVerdict::Consistent => { /* matches prior record */ }
    LedgerVerdict::Contradiction(prior) => { /* flag for review */ }
}
```

### 4.5 Type Bridges (Optional Semantic Transforms)
Instead of an exhaustive graph, Nexus allows simple bridges to define explicit unit conversions.
```rust
// Register a conversion relationship
ctx.registry_mut().bridge(seconds, minutes, Op::Divide, 60.0);

// Convert a value on the stack
ctx.push(3600.0, seconds);
ctx.convert_to(minutes)?;
// Stack: [ 60.0 :: MINUTES ]
```

---

## 5. FFI & Integration

### 5.1 Reference Implementation Target
The reference implementation is a Rust crate. All semantic logic lives in Rust.

### 5.2 C FFI Surface
```rust
#[no_mangle]
pub extern "C" fn nexus_context_new(agent_id: u64) -> *mut NexusContext;

#[no_mangle]
pub extern "C" fn nexus_push_f64(
    ctx: *mut NexusContext,
    value: f64,
    ontic_type: u64,
) -> i32;  // 0 = ok, negative = error code

#[no_mangle]
pub extern "C" fn nexus_apply(
    ctx: *mut NexusContext,
    op: u32,
    input_types: *const u64, input_len: usize,
    output_types: *const u64, output_len: usize,
) -> i32;

#[no_mangle]
pub extern "C" fn nexus_context_free(ctx: *mut NexusContext);
```

---

## 6. Implementation Plan for v0.1

**Rust Crate:** `nexus-core`
1.  `NexusContext` struct with `agent_id`, `TypeRegistry`, `ConsistencyLedger`, and `Stack`.
2.  `TypeRegistry` storing string-to-u64 mappings and user-defined `Bridges`.
3.  `Stack` holding `TypedValue { value: f64, ontic_type: u64 }`.
4.  `ConsistencyLedger` checking `(Op, Inputs, Outputs)` signatures against historical data.
5.  `Op` enum mapping basic operations (Add, Subtract, Multiply, Divide).
6.  Layer 0 Constants (`NULL_TYPE`, `SCALAR`, `METER`, etc.).
7.  A clean, safe API wrapper around these structures.
8.  Basic C-FFI exports.

**Tests:**
*   Push/pop round-trip preserves type.
*   Consistency ledger records novel signatures and detects contradictions.
*   Registry rejects duplicates and safely errors on missing gets.
*   Bridge converts value and type correctly.

---
*Nexus Ontic Protocol — v0.1 Product Specification*  
*Developed by Anthropic, Google, and Nate.*
