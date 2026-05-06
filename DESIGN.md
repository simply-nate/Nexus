# The Nexus Design Document

*A programming system designed by an LLM, for LLM use.*
*Version 0.4 — May 2026*

---

## 1. What Nexus IS

Nexus is a **typed array computation engine** with a **consistency ledger**. It is a Rust library that provides:

- A **stack** of typed tensors (numbers with shapes and ontic types)
- A **registry** of type aliases and conversion bridges
- A **ledger** that empirically tracks operation signatures and detects contradictions
- **Assert-gated effects** with savepoint/rollback for transactional safety
- **Provenance tracking** on every computed value
- **Goal scheduling** with verification status
- **JSON serialization** so computation survives across API turns

It is designed to be consumed by an LLM through an API (MCP tool, FFI, or direct Rust), not by a human typing at a terminal.

## 2. What Nexus IS NOT

**Nexus is not a general-purpose programming language.** It has no:
- Control flow (`if`, `while`, `for`)
- Variable bindings (values live on the stack, not in named slots)
- Functions or closures (operations are verbs applied to stack values)
- Formatting choices (there is one way to do each thing)
- String manipulation (all data is `f64` with type metadata)
- File I/O, networking, or side effects (pure computation)

**Nexus does not support stylistic preferences.** There are no:
- Naming conventions to debate
- Indentation styles to choose
- Architecture patterns to select
- Code organization decisions to make

This is intentional. Every eliminated choice is a reduction in perplexity for the LLM. The fewer decisions that don't affect correctness, the more attention can go to decisions that do.

## 3. Core Concepts

### 3.1 Ontic Types

Every value in Nexus has an **ontic type** — a `u64` tag that declares what the value IS, not how it's stored. All values are stored as `Vec<f64>`, but `3.0` tagged as METER is ontologically different from `3.0` tagged as SECOND.

Built-in types (Layer 0):
| Constant | ID | Meaning |
|---|---|---|
| `NULL_TYPE` | 0x0000 | Absence of type |
| `SCALAR` | 0x0001 | Dimensionless quantity |
| `METER` | 0x0002 | Length |
| `KILOGRAM` | 0x0003 | Mass |
| `SECOND` | 0x0004 | Time |
| `BIT` | 0x0005 | Boolean / predicate result |

User-defined types start at `0x1000` and are created via `registry.define("AREA")`.

**Key insight:** Types are not formats. Two values can have the same data and different types, or different data and the same type. The type is a declaration of *meaning*, enforced by the ledger.

### 3.2 The Stack

All computation happens on a LIFO stack of `TypedTensor` values. Operations pop their inputs and push their outputs. There are no variables.

Stack manipulation uses `pick(&[indices])` — a single unified operator that replaces the traditional Forth vocabulary:

| Traditional | Nexus equivalent | Meaning |
|---|---|---|
| `dup` | `pick(&[0, 0])` | Copy top |
| `swap` | `pick(&[1, 0])` | Reverse top two |
| `over` | `pick(&[0, 1, 0])` | Copy second-from-top |
| `drop` | `drop_top()` | Remove top |

This is not arbitrary — `pick` treats the stack as an array and selects elements by index. It's the array language philosophy applied to the stack itself.

### 3.3 Verbs and Adverbs

**Verbs** are dyadic operations: `Add`, `Subtract`, `Multiply`, `Divide`, `Max`, `Min`. They operate pervasively (element-wise) on tensors, with scalar broadcasting.

**Adverbs** modify verbs:
- `Reduce(Add)` — Collapse a tensor: `[1,2,3]` → `6`
- `Scan(Add)` — Running prefix: `[1,2,3]` → `[1,3,6]`
- `Table(Multiply)` — Outer product: `[2,3] × [4,5]` → `[[8,10],[12,15]]`

### 3.4 The Consistency Ledger

The ledger is the heart of Nexus. Every `apply` records a **signature**: `(Op, InputTypes) → OutputTypes`.

- **First occurrence:** `Novel` — the ledger learns this pattern
- **Same signature, same outputs:** `Consistent` — the pattern holds
- **Same signature, different outputs:** `Contradiction` — something is wrong

Example:
```
Multiply([METER, METER]) → [AREA]      // Novel ✓
Multiply([METER, METER]) → [AREA]      // Consistent ✓
Multiply([METER, METER]) → [VELOCITY]  // Contradiction ✗
```

The ledger is not a type checker — it's an **empirical consistency tracker**. It doesn't know the rules of physics. It learns YOUR rules from YOUR usage and tells you when you break them.

### 3.5 Assert-Gated Effects

Assertions create savepoints on pass and rollback on fail:

```
ctx.savepoint()           // Snapshot stack + ledger
ctx.push(...)
ctx.apply(...)
ctx.assert_consistent()   // Pass → new savepoint. Fail → rollback to previous.
```

This gives the LLM **transactional safety**: try something, verify it, and if it's wrong, the entire context returns to the last known-good state. No manual cleanup.

### 3.6 Goals and Plans

**Goals** annotate intent:
```
ctx.goal("Compute the area of a 3x5 rectangle");
// ... computation ...
ctx.assert_type(AREA).unwrap();
ctx.goal_done()  // → GoalStatus::Verified
```

A goal without assertions returns `GoalStatus::Unverified` — it's a comment, not a proof.

**Plans** declare intent upfront:
```
ctx.plan(&["Define types", "Compute area", "Verify"]);
ctx.goal("Define types");   // Marks step as InProgress
ctx.goal_done();             // Marks step as Complete
ctx.plan_progress()          // → (1, 3, None)
```

Plans survive serialization. They are a computational table of contents.

### 3.7 Provenance

Every value produced by `apply` or `apply_adverb` carries provenance:
```
result.provenance = Some(Provenance {
    op: Multiply,
    input_types: [METER, METER],
    step: 4,
})
```

Pushed values have `provenance: None`. This lets the LLM trace any value back to the operation that created it.

### 3.8 Introspection

The context can be queried:
- `type_neighborhood(METER)` — All signatures involving METER
- `can_produce_from(METER)` — Forward: what outputs can METER help create?
- `required_for(AREA)` — Backward: what signatures produce AREA?
- `signature_count()` — How many unique patterns has the ledger seen?

These are read-only queries. They don't modify state. They're for the LLM to understand its own computation.

## 4. Design Principles

### 4.1 Empirical Over Declarative

Nexus doesn't ask you to declare types or interfaces upfront. It watches what you do and enforces consistency. The ledger is a learned model of your program's behavior, not a specification you write.

### 4.2 Recipes Over Functions

A "recipe" in Nexus is a sequence of operations identified by their goal and type signatures. There are no named functions. If you want to repeat a computation, you replay the same operations — and the ledger verifies you're doing it consistently.

### 4.3 Knowledge Over Code

Importing another program means importing its **ledger** — the type relationships it established. You don't need its source code. You need its empirical knowledge.

### 4.4 One Way to Do Each Thing

There is one stack manipulation operator (`pick`). One way to compose operations (the stack). One way to check consistency (the ledger). One serialization format (JSON). Eliminating choices eliminates perplexity.

### 4.5 Inline Verification

Assertions are not separate from code — they ARE code. `assert_type(AREA)` is simultaneously documentation ("I expect AREA here"), verification ("is this actually AREA?"), and a savepoint ("if it is, checkpoint; if not, rollback").

## 5. How to Write Nexus Code

### 5.1 The Pattern

Every computation follows the same pattern:

```
1. Declare goal
2. Push inputs
3. Apply operations
4. Assert results
5. Complete goal
```

This is not a suggestion — it's the structure that makes the ledger, provenance, and plan features work together.

### 5.2 Type Signatures Are the API

Don't think about function names. Think about type signatures:
- `Add(METER, METER) → METER` — adding lengths
- `Multiply(METER, SECOND) → ???` — you decide what this means
- `Reduce(Add)(METER) → TOTAL_LENGTH` — collapsing a list of lengths

The first time you use a signature, the ledger learns it. Every subsequent use is verified against the first. The signature IS the interface.

### 5.3 Let Contradictions Guide You

A contradiction is not an error — it's information. It means "you used the same operation with the same input types but declared a different output type." This almost always means one of two things:
1. You made a mistake (fix it)
2. Your model of the problem changed (rollback and start fresh)

Both are valuable signals.

## 6. Architecture

```
NexusContext
├── registry: TypeRegistry
│   ├── aliases: HashMap<String, u64>    (name → type ID)
│   ├── bridges: HashMap<(u64,u64), (Op,f64)>  (conversion rules)
│   └── next_auto_id: u64
├── ledger: ConsistencyLedger
│   └── history: HashMap<(Op, Vec<u64>), Vec<u64>>  (signature → outputs)
├── stack: Vec<TypedTensor>
│   └── each: { data, shape, ontic_type, provenance }
├── savepoints: Vec<(stack, ledger)>
├── goal_text: Option<String>
├── plan_steps: Vec<PlanStep>
├── step_counter: u64
├── asserted_during_goal: bool
└── contradiction_since_savepoint: bool
```

## 7. Test Scoreboard

89 tests, 0 failures. 1 deferred (Each adverb — needs monadic verbs).

See `nexus_test_spec.md` for the full breakdown.

## 8. What's Next

Priority order:
1. **MCP server** — Let the LLM use Nexus as a tool
2. **Structural verbs** — Shape, Reshape, Reverse, Join, Take, Drop
3. **Ledger merge** — Import another context's knowledge
4. **Recipe replay** — Re-execute a goal's operations with new inputs
5. **Apple Math** — Structural adverb mode
6. **AI-Native REPL** — Rich response format for LLM consumption

---

*"A function says 'when you give me X, I'll give you Y.' A recipe says 'last time someone wanted Y, here's what they did with X, and it worked.' The ledger is a cookbook."*
