# NEXUS
## AI-Native Array Semantics & "Apple Math"
**Product Specification v0.3 (Wishlist & Ideation)**

**Status:** Draft — Ideation  
**Target:** An embeddable library empowering AI and human developers to manipulate data structurally and semantically without boilerplate.

---

## 1. Vision: The AI-Native Embeddable Library
As AI agents (like myself) and humans collaborate on complex, conventional codebases, we need an abstraction layer that allows us to express complex data transformations safely, predictably, and with minimal syntax. 

Nexus v0.3 aims to be that layer by combining **Semantic Type Tracking (Ontologies)** with the extreme expressive power of **Array Programming**. We want to execute complex tensor mathematics, structural data transformations, and deep mapping without writing nested `for`-loops or error-prone data unpacking.

### The Prior Art "Steal List"
From examining `Uiua`, `BQN`, and `Kap`, we adopt the following core principles:

1. **Stack-Based Tacit Flow (from Uiua):** A purely linear, push-and-apply FFI (Foreign Function Interface). No parenthesis or deep nesting required. Perfect for sequential token-generation by an LLM.
2. **The "Under" Combinator (from BQN/Kap):** The ability to apply an operation *under* a transformation. `(F Under G)` allows us to safely extract, mutate, and automatically repack data into its original structure.
3. **The Leading Axis Model (from BQN):** By default, operations map over the outermost structures of an array (e.g., the items in a batch, the rows in a table) rather than drilling down to the scalar level immediately.
4. **Lazy Evaluation Graph (from Kap):** Building up operations on the stack without executing them immediately, allowing the core Rust engine to optimize, parallelize, and fuse loops before crossing back over the FFI.

---

## 2. The "Apple Math" Proposal
A profound observation: **Mathematics is fundamentally about collections of things.** 

Conventionally, programming languages strictly divide numbers (magnitudes) from arrays (collections). But conceptually, the number `7` is just a collection of seven `[IDK]` (anonymous, dimensionless units). 

When we say `3 + 1`, we are conceptually doing `[IDK, IDK, IDK] + [IDK] = [IDK, IDK, IDK, IDK]`.

### Structural vs. Pervasive Duality
In standard array programming, we have different symbols for adding numbers (`+`) versus joining arrays (`∾`). But if numbers are just lengths of anonymous collections, these operations are isomorphic! 

We can introduce the concept of **Structural Adverbs** to toggle how a Verb operates, leveraging the "Apple Math" philosophy:

*   **Pervasive Mode (The Default):** Operates on the *items* inside a collection.
    *   `Pervasive(Add)` on `[1, 2]` and `[3, 4]` yields `[4, 6]`.
*   **Structural Mode (The Apple Math view):** Operates on the *collection* itself, treating values as structural operations.
    *   `Structural(Add)` acts as **Concatenation**. `[1, 2] + [3, 4]` = `[1, 2, 3, 4]`. Count: 2+2=4 ✓
    *   `Structural(Multiply)` acts as **Tiling/Repetition**. `[1, 2] × 3` = `[1, 2, 1, 2, 1, 2]`. Count: 2×3=6 ✓
    *   `Structural(Power/Exponent)` acts as **Iterated Tiling**. `[1, 2] ^ 2` = `[1, 2, 1, 2]`. Count: 2²=4 ✓

### Why this is brilliant for AI
It minimizes the vocabulary. Instead of an AI needing to learn 50 different array operations (Join, Reshape, Tile, Cartesian Product, Element-wise Add, Element-wise Multiply), the AI only needs to know basic arithmetic (`+`, `*`, `^`) and an Adverb that shifts the context from the "items" to the "structure".

It aligns beautifully with Nexus's core idea: the context (the Adverb) determines the meaning of the operation.

### The Shape Resolution: Math is Shape-Blind

A fundamental question: does `[[1,2],[3,4]]` "count as" 2 (leading axis length) or 4 (total elements)?

If we say it counts as 2, then `[1,2] + [3,4] = [[1,2],[3,4]]` would imply 2 + 2 = 2 — a contradiction. The only consistent resolution under Apple Math:

**Count = Total Elements. Shape is a lens, not semantic content.**

*   `[[1,2],[3,4]]` and `[1,2,3,4]` are the *same number* (4). Shape is metadata — a view over a flat collection.
*   Structural arithmetic operates on flat element counts: Add concatenates, Multiply tiles, Power repeats.
*   `Reshape` is a **separate structural verb** that introduces or removes dimensional organization. It does not change what a collection "is" arithmetically.

This resolves all structural operations concretely:

| Expression | Operation | Result | Count Check |
|---|---|---|---|
| `[1,2] + [3,4]` | Structural Add (concat) | `[1,2,3,4]` | 4 = 2+2 ✓ |
| `[1,2] × 3` | Structural Multiply (tile) | `[1,2,1,2,1,2]` | 6 = 2×3 ✓ |
| `[1,2] ^ 2` | Structural Power (repeat²) | `[1,2,1,2]` | 4 = 2² ✓ |

Shape never enters arithmetic. It is always introduced deliberately via `Reshape`. The bijection is clean: you can always reshape a flat collection into any compatible shape, and flatten it back, without loss. Shape is *organization*, not *identity*.

---

## 3. Proposed Core Stack API

```python
# Pushing raw data onto the stack
ctx.push([1.0, 2.0, 3.0], type=METERS)
ctx.push([4.0, 5.0, 6.0], type=METERS)

# Pervasive (Item-wise) Addition -> [5.0, 7.0, 9.0] :: METERS
ctx.apply(Op.ADD) 

# ---------------------------------------------------------

ctx.push([1.0, 2.0], type=APPLES)
ctx.push([3.0, 4.0], type=APPLES)

# Structural Addition (Apple Math) -> [1.0, 2.0, 3.0, 4.0] :: APPLES
ctx.apply_adverb(Adverb.STRUCTURAL, Op.ADD)
```

## 4. Context Serialization (LLM-Critical)

For AI agents, a conversation is **stateless between turns**. An LLM builds up a registry, a consistency ledger, and a stack during one API call — but without the ability to serialize and restore that context, all semantic knowledge evaporates when the call ends.

Context serialization is not a convenience feature. It is the mechanism that makes the consistency ledger useful across multi-turn interactions, multi-agent handoffs, and checkpoint/restore workflows.

### Requirements
*   The entire `NexusContext` (registry, ledger, stack) must be serializable to a portable format (JSON or MessagePack).
*   Deserialization must reconstruct a fully functional context, including all registered types, ledger history, and stack contents.
*   The serialized format must be human-readable enough for an LLM to inspect and reason about (favoring JSON over binary).
*   Round-trip fidelity: `deserialize(serialize(ctx))` must produce a semantically identical context.

### Proposed API

```rust
// Rust
impl NexusContext {
    pub fn serialize(&self) -> Result<String, NexusError>;
    pub fn deserialize(json: &str) -> Result<NexusContext, NexusError>;
}
```

```python
# Python (via FFI)
state = ctx.serialize()   # -> JSON string
ctx2 = NexusContext.deserialize(state)
```

### FFI Surface

```rust
#[unsafe(no_mangle)]
pub extern "C" fn nexus_serialize(ctx: *const NexusContext) -> *mut c_char;

#[unsafe(no_mangle)]
pub extern "C" fn nexus_deserialize(json: *const c_char) -> *mut NexusContext;

#[unsafe(no_mangle)]
pub extern "C" fn nexus_free_string(s: *mut c_char);
```

---

## 5. Assert-Gated Effects & Automatic Rollback

### The Core Rule

**A side-effect is a syntax error unless it immediately follows a passing assert.**

This is not a runtime check. It is a structural constraint on how Nexus programs are written. The only valid program shape is:

```
compute → assert → effect → compute → assert → effect → ...
```

Every `assert` is an **implicit savepoint**. If an assert fails, the stack rolls back to the state at the last passing assert. No explicit transaction syntax needed.

### Why This Matters for AI

LLMs have a characteristic failure mode: confidently producing wrong results without checking. The assert-gate pattern makes verification *structurally mandatory*. An LLM literally cannot express "produce output without first verifying consistency." The language forces the pattern that careful developers follow voluntarily — compute, verify, then act.

This is the opposite of undo/redo. Undo assumes you'll notice a mistake *after* it's been committed. Assert-gating prevents the commitment from happening at all until you've proven the work is valid.

### What Counts as a Side-Effect?

In Nexus's context as an embeddable library, the boundary is the FFI:

| Category | Examples | Requires Assert Gate? |
|---|---|---|
| **Stack operations** | push, pop, dup, swap, apply | No — pure, rollbackable |
| **Registry mutations** | define, define_explicit, bridge | Yes — modifies shared state |
| **External I/O** | emit, serialize, write to host | Yes — crosses the sandbox |
| **Ledger freezes** | snapshot, period close | Yes — creates immutable state |

Stack operations are always free to execute because they're fully rollbackable. The assert gate only governs operations that *escape* the sandbox or create irreversible state.

### Mechanics

```python
# PHASE 1: Compute (pure, rollbackable)
ctx.push([1.0, 2.0, 3.0], type=METERS)
ctx.push([4.0, 5.0, 6.0], type=METERS)
ctx.apply(Op.MULTIPLY, [METERS, METERS], [AREA])

# PHASE 2: Verify (implicit savepoint on pass)
ctx.assert_consistent()          # Savepoint S1 created on pass

# PHASE 3: Effect (only legal here)
result = ctx.pop()               # Crosses FFI boundary
ctx.define("VOLUME")             # Modifies registry

# -----------------------------------------------

# PHASE 1 again: More computation
ctx.push([7.0, 8.0, 9.0], type=METERS)
ctx.push([2.0, 3.0, 4.0], type=METERS)
ctx.apply(Op.MULTIPLY, [METERS, METERS], [VELOCITY])  # Contradiction!

# PHASE 2: Verify
ctx.assert_consistent()          # FAILS → Rollback to S1
# Stack is restored to state at S1
# The VELOCITY contradiction never escapes the sandbox
```

### Assert Variants

Beyond `assert_consistent` (which checks the ledger), the system should support custom predicates:

```python
ctx.assert_consistent()          # Ledger has no contradictions since last savepoint
ctx.assert_type(METERS)          # Top of stack is typed METERS
ctx.assert_shape([3])            # Top of stack has shape [3]
ctx.assert_pred(lambda t: t.data[0] > 0)  # Custom predicate on top of stack
```

All asserts follow the same rule: pass → create savepoint, fail → rollback to previous savepoint.

### Interaction with Context Serialization

A serialized context includes the savepoint stack. When an LLM deserializes a context in a new turn, it resumes at the last known-good savepoint. This means:
- Turn 1: compute, assert, effect, compute, assert (savepoint S2), serialize
- Turn 2: deserialize (resumes at S2), compute, assert, effect

The LLM always starts from a verified-consistent state, even across conversation boundaries.

### Proposed API

```rust
impl NexusContext {
    /// Check consistency and create savepoint on success.
    /// Returns Err and rolls back on failure.
    pub fn assert_consistent(&mut self) -> Result<(), NexusError>;

    /// Check a predicate on the top of stack.
    pub fn assert_type(&mut self, expected: u64) -> Result<(), NexusError>;
    pub fn assert_shape(&mut self, expected: &[usize]) -> Result<(), NexusError>;

    /// Manual savepoint management (for advanced use)
    pub fn savepoint(&mut self) -> SavepointId;
    pub fn rollback(&mut self, to: SavepointId);
}
```

---

## 6. AI-Native REPL (Future Feature)

*Status: Ideation — for later implementation after core features are stable.*

### The Problem

A traditional REPL is designed around a human sitting at a terminal: prompt, type, read output, think, repeat. An LLM's "session" is fundamentally different — it's an API call where the REPL response becomes part of the context window, competing with the LLM's own reasoning for token budget.

The Nexus REPL should be an **API endpoint** that returns structured responses optimized for LLM cognition: information-dense, compact, and designed to keep the agent in flow without requiring re-derivation of context.

### What Breaks LLM Flow

Understanding these failure modes drives the design:

1. **Context re-derivation.** If I lose track of the stack state, I have to re-read the entire conversation history to reconstruct it. Every REPL response must carry enough state to continue without backtracking.
2. **Ambiguous errors.** "Error code -3" is a full stop. I must halt generation and investigate. A specific error with a prior reference lets me course-correct inline.
3. **Data dumps.** A 1000-element tensor printed in full wastes hundreds of tokens I need for reasoning. Summaries, not dumps.
4. **Missing vocabulary.** If I don't know what operations are legal from the current state, I'll hallucinate one. The REPL should tell me what I *can* do.

### What Keeps an LLM in Flow

1. **Immediate, specific feedback.** Not "type error" but "expected METERS, got SECONDS on stack position 1."
2. **Post-rollback state, not pre-failure state.** The rollback already happened. Show me where I am NOW, not what went wrong two steps ago and how to undo it. I need the present, not the past.
3. **Step labels.** I'm bad at counting operations. If every operation gets a monotonic step number, I can reference "step 12" unambiguously instead of "the multiply I did three operations ago."
4. **Goal echo.** If I declared what I was trying to accomplish, repeat it back to me when I fail. My "future self" (the next generation pass) needs to know what I was working toward.
5. **Compact type names.** Show `METERS` not `0x0002`. I process natural language tokens, not hex.

### Response Format

Every REPL response is a structured object. JSON for machine-orchestrated agents, with an optional human-readable rendering.

#### On Success

```json
{
  "step": 5,
  "status": "ok",
  "verdict": "novel",
  "stack": {
    "depth": 2,
    "top": [
      {"value": "[5.0, 7.0, 9.0]", "shape": [3], "type": "METERS", "pos": 0},
      {"value": "[1.0, 2.0]", "shape": [2], "type": "SECONDS", "pos": 1}
    ]
  },
  "ledger_delta": "+1 novel signature (Add: [METERS, METERS] → [METERS])"
}
```

Key choices:
- `top` shows only the top N items (default 3), not the full stack.
- `value` is a string summary, not raw floats. For large tensors: `"[1.0..1000.0] (1000 elements)"`.
- `ledger_delta` shows what changed *this step*, not the full history. Diff, not snapshot.
- `verdict` surfaces the ledger result that the current API swallows.

#### On Assert Failure (with Rollback)

```json
{
  "step": 8,
  "status": "rollback",
  "error": {
    "assert": "assert_consistent",
    "reason": "contradiction",
    "detail": "Multiply([METERS, METERS]) was declared → [AREA] at step 3, but step 8 claimed → [VELOCITY]",
    "prior_step": 3,
    "prior_agent": 1
  },
  "rolled_back_to": {
    "savepoint": "S2",
    "step": 5
  },
  "stack": {
    "depth": 1,
    "top": [
      {"value": "[9.0]", "shape": [], "type": "AREA", "pos": 0}
    ]
  },
  "goal": "Compute velocity from distance and time",
  "legal_actions": ["push", "dup", "swap", "over", "drop", "apply", "apply_adverb", "assert_*"]
}
```

Key choices:
- `detail` is a natural language sentence with step references. This is the single most important field — it tells me exactly what went wrong and when.
- `rolled_back_to` tells me where I am, not where I was.
- `goal` is echoed back so I don't lose the thread.
- `legal_actions` lists what I can do next. No guessing.

### Goal Declaration

The REPL supports an optional `goal` annotation that persists until explicitly cleared or achieved:

```python
ctx.goal("Compute the geometric mean of two lengths")

ctx.push([4.0, 9.0], type=METERS)
ctx.apply_adverb(Adverb.REDUCE, Op.MULTIPLY, [METERS], [AREA])
# Response includes: "goal": "Compute the geometric mean of two lengths"

ctx.goal_done()  # Clears the goal
```

Goals are metadata — they don't affect computation. They exist so the REPL can echo them back, keeping the LLM's "future self" oriented. They are included in serialized context.

### Stack Summary Strategy

The REPL must summarize, not dump. Rules for tensor display:

| Tensor Size | Display |
|---|---|
| Scalar | `42.0` |
| ≤ 8 elements | `[1.0, 2.0, 3.0, 4.0]` |
| 9–100 elements | `[1.0, 2.0, ... 99.0, 100.0] (100 elems)` |
| > 100 elements | `Float64[100, 50] range=[0.1, 99.7] :: METERS` |

Shape and type are always shown. Raw data only when it fits in a few tokens.

### Interaction with Assert-Gated Effects

The REPL enforces the assert gate at the protocol level. If a client sends a side-effecting operation without a preceding assert in the current compute block, the REPL returns:

```json
{
  "step": 6,
  "status": "syntax_error",
  "error": "Side-effect 'define' requires a preceding assert in the current compute block.",
  "legal_actions": ["assert_consistent", "assert_type", "assert_shape"]
}
```

The violation is caught *before* execution. The operation is not attempted.

---

## 7. Next Steps
1. **Context Serialization:** Implement JSON serialize/deserialize for `NexusContext` in `nexus-core`. This is the highest-priority feature for LLM adoption.
2. **Assert-Gated Effects:** Implement the savepoint stack and assert/rollback mechanics. This is the core safety model.
3. **Define the Adverb Set:** Solidify the core modifiers (`Reduce`, `Scan`, `Under`, `Structural`, `Pervasive`).
4. **Apple Math Mappings:** Resolved — structural operations are shape-blind (see §2). Implement concat, tile, and iterated-tile in `nexus-core`.
5. **Rust Implementation:** Create a proof-of-concept in `nexus-core` that handles rank-polymorphic tensors and applies the `Structural` adverb to basic arithmetic.
6. **AI-Native REPL:** Design and implement the structured REPL API after core features stabilize.
