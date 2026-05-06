# NEXUS — Test Specification & Implementation Status

**Last Updated:** 2026-05-05  
**Purpose:** Living document tracking every testable behavior across all spec versions. Check boxes indicate implementation status.

---

## How to Read This Document

- ✅ **Implemented & Tested** — Rust code exists and a passing test covers it.
- ⚠️ **Implemented, No Test** — Code exists but no test verifies it.
- ❌ **Not Implemented** — Spec'd but no code exists.
- 🔮 **Future** — Spec'd in v0.3 wishlist, not yet targeted for implementation.

Each test case has a **Test ID** (e.g. `v1.reg.01`) for unambiguous reference.

---

## v0.1 — Core Ontic Protocol

### Type Registry

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v1.reg.01` | Define a new type by alias | `registry.define("AREA")` returns a `u64` ≥ `0x1000` | ⚠️ Code exists, no test |
| `v1.reg.02` | Define with explicit ID | `registry.define_explicit("PESOS", 0x2001)` succeeds | ⚠️ Code exists, no test |
| `v1.reg.03` | Reject duplicate alias | Second `define("AREA")` returns `Err(TypeAlreadyExists)` | ⚠️ Code exists, no test |
| `v1.reg.04` | Get existing type | `registry.get("METER")` returns `0x0002` | ⚠️ Code exists, no test |
| `v1.reg.05` | Get missing type errors | `registry.get("NONEXISTENT")` returns `Err(TypeNotFound)` | ⚠️ Code exists, no test |
| `v1.reg.06` | Layer 0 constants present | `NULL_TYPE`, `SCALAR`, `METER`, `KILOGRAM`, `SECOND`, `BIT` all resolvable | ⚠️ Code exists, no test |
| `v1.reg.07` | Auto-ID increments | Two sequential `define()` calls return consecutive IDs | ⚠️ Code exists, no test |
| `v1.reg.08` | Explicit ID adjusts auto counter | After `define_explicit(_, 0x5000)`, next `define()` returns `0x5001` | ⚠️ Code exists, no test |

### Stack Operations

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v1.stk.01` | Push and pop scalar | Push `42.0 :: SCALAR`, pop returns same value and type | ✅ `test_push_pop` |
| `v1.stk.02` | Pop empty stack errors | `pop()` on empty stack returns `Err(StackUnderflow)` | ⚠️ Code exists, no test |
| `v1.stk.03` | Stack is LIFO | Push A then B, pop returns B then A | ❌ No test |
| `v1.stk.04` | Multiple items on stack | Push 3 items, verify depth by popping all 3 | ❌ No test |

### Apply (Dyadic Operations)

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v1.app.01` | Add two scalars | `3.0::M + 4.0::M → 7.0::M` with explicit type sig | ❌ No test (tensor version exists) |
| `v1.app.02` | Multiply two scalars | `3.0::M * 3.0::M → 9.0::AREA` with explicit type sig | ❌ No test (tensor version exists) |
| `v1.app.03` | Divide with zero errors | `1.0 / 0.0` returns `Err(ExecutionError)` | ⚠️ Code exists, no test |
| `v1.app.04` | Type mismatch errors | Push `METER`, apply with `expected_inputs=[SECOND]` → error | ⚠️ Code exists, no test |
| `v1.app.05` | Stack underflow on apply | Apply requiring 2 inputs with only 1 on stack → error | ⚠️ Code exists, no test |

### Consistency Ledger

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v1.led.01` | Novel signature recorded | First `Multiply([M,M] → [AREA])` returns `Novel` | ⚠️ Code exists, no test |
| `v1.led.02` | Consistent re-use | Second identical signature returns `Consistent` | ⚠️ Code exists, no test |
| `v1.led.03` | Contradiction detected | `Multiply([M,M] → [AREA])` then `Multiply([M,M] → [BIRDS])` returns `Contradiction` | ⚠️ Code exists, no test |
| `v1.led.04` | Contradiction includes prior sig | The `Contradiction` variant contains the original signature for comparison | ⚠️ Code exists, no test |
| `v1.led.05` | Different ops don't interfere | `Add([M,M] → [M])` and `Multiply([M,M] → [AREA])` are both `Novel` | ❌ No test |

### Type Bridges

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v1.brg.01` | Register and use bridge | Bridge `SECONDS → MINUTES` via `Divide, 60.0`. Push `3600.0::SEC`, convert, get `60.0::MIN` | ⚠️ Code exists, no test |
| `v1.brg.02` | Convert same type is no-op | `convert_to(METERS)` when top is already `METERS` → unchanged | ⚠️ Code exists, no test |
| `v1.brg.03` | Missing bridge errors | `convert_to` with no registered bridge → `Err(ConversionNotFound)` | ⚠️ Code exists, no test |
| `v1.brg.04` | Bridge applies to tensors | Bridge converts all elements of a vector, not just the first | ⚠️ Code exists, no test |

### C-FFI Surface

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v1.ffi.01` | Create and free context | `nexus_context_new` / `nexus_context_free` round-trip without crash | ⚠️ Code exists, no test |
| `v1.ffi.02` | Push tensor via FFI | `nexus_push_tensor` with valid data returns `0` | ⚠️ Code exists, no test |
| `v1.ffi.03` | Apply via FFI | `nexus_apply` returns `0` on success | ⚠️ Code exists, no test |
| `v1.ffi.04` | Null context returns -1 | All FFI functions return `-1` on null ctx pointer | ⚠️ Code exists, no test |
| `v1.ffi.05` | Pop via FFI | Pop result back across FFI boundary | ❌ **FFI function missing** |
| `v1.ffi.06` | Convert via FFI | `nexus_convert_to` exposed in FFI | ❌ **FFI function missing** |
| `v1.ffi.07` | Ledger verdict via FFI | Apply returns structured verdict, not just 0/-3 | ❌ **FFI function missing** |

---

## v0.2 — Array Semantics Extension

### Tensor Operations

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v2.ten.01` | Element-wise multiply vectors | `[2,3,4]::M * [5,5,5]::M → [10,15,20]::AREA` | ✅ `test_tensor_math` |
| `v2.ten.02` | Scalar-tensor broadcast | `[1,2,3]::M * 2.0::SCALAR → [2,4,6]::DM` | ✅ `test_scalar_broadcast` |
| `v2.ten.03` | Tensor-scalar broadcast | `2.0::SCALAR * [1,2,3]::M → [2,4,6]::DM` (reversed order) | ❌ No test |
| `v2.ten.04` | Shape mismatch errors | `[1,2,3] + [1,2]` → `Err(ShapeMismatch)` | ⚠️ Code exists, no test |
| `v2.ten.05` | All pervasive math verbs | Add, Subtract, Multiply, Divide, Max, Min on matching vectors | ❌ Only Multiply tested |
| `v2.ten.06` | NumPy-style broadcasting | `[3] + [3,1]` → trailing dimension alignment & expansion | ❌ **Not implemented** |
| `v2.ten.07` | Multi-dim tensor operations | `[2,3] * [2,3]` matrix element-wise | ❌ No test |

### Structural Verbs

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v2.str.01` | Shape (△) | Push tensor, get its shape as a new SCALAR tensor | ❌ Not implemented |
| `v2.str.02` | Reshape (↯) | Reshape `[1,2,3,4,5,6]` with shape `[2,3]` | ❌ Not implemented |
| `v2.str.03` | Reverse (⇌) | Reverse `[1,2,3]` → `[3,2,1]`, preserves type | ❌ Not implemented |
| `v2.str.04` | Join (⊂) | Concatenate `[1,2]::M` and `[3,4]::M` → `[1,2,3,4]::M` | ❌ Not implemented |
| `v2.str.05` | Join type mismatch errors | Join `[1,2]::M` and `[3,4]::SEC` → error | ❌ Not implemented |
| `v2.str.06` | Take (↙) | Take first 2 of `[1,2,3,4]` → `[1,2]` | ❌ Not implemented |
| `v2.str.07` | Drop (↘) | Drop first 2 of `[1,2,3,4]` → `[3,4]` | ❌ Not implemented |

### Adverbs

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v2.adv.01` | Reduce(Add) | `[1,2,3,4]::M` → `10.0::TOTAL` | ✅ `test_adverb_reduce` |
| `v2.adv.02` | Reduce(Multiply) | `[2,2,2]::M` → `8.0::VOL` | ⚠️ Code exists, no test |
| `v2.adv.03` | Reduce on empty errors | Reduce on `[]` → `Err(ExecutionError)` | ⚠️ Code exists, no test |
| `v2.adv.04` | Scan(Add) | `[1,2,3]::SEC` → `[1,3,6]::SEC` | ❌ **Not implemented** |
| `v2.adv.05` | Each | Apply a scalar verb to every element individually | ❌ **Not implemented** |
| `v2.adv.06` | Table (Outer Product) | `[2,3]::M × [4,5]::M` → `[[8,10],[12,15]]::AREA` | ❌ **Not implemented** |

### Stack Primitives (from Forth)

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v2.stk.01` | dup | Duplicate top of stack | ❌ Not implemented |
| `v2.stk.02` | swap | Swap top two items | ❌ Not implemented |
| `v2.stk.03` | over | Copy second item to top | ❌ Not implemented |
| `v2.stk.04` | rot | Rotate top three items | ❌ Not implemented |
| `v2.stk.05` | drop | Discard top of stack | ❌ Not implemented |

---

## v0.3 — Wishlist Features

### Apple Math (Structural Adverb)

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v3.apl.01` | Structural Add (concat) | `[1,2]::A + [3,4]::A → [1,2,3,4]::A` | 🔮 Future |
| `v3.apl.02` | Structural Multiply (tile) | `[1,2]::A × 3 → [1,2,1,2,1,2]::A` | 🔮 Future |
| `v3.apl.03` | Structural Power (iterated tile) | `[1,2]::A ^ 2 → [1,2,1,2]::A` | 🔮 Future |
| `v3.apl.04` | Pervasive is default | `apply(Add)` without adverb is element-wise | 🔮 Future |
| `v3.apl.05` | Shape-blindness | `count([1,2,3,4]) == count([[1,2],[3,4]])` (both 4) | 🔮 Future |
| `v3.apl.06` | Structural type consistency | Structural Add on mismatched types → error | 🔮 Future |

### Context Serialization

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v3.ser.01` | Serialize empty context | New context serializes to valid JSON | ❌ Not implemented |
| `v3.ser.02` | Round-trip empty context | `deserialize(serialize(ctx))` produces identical context | ❌ Not implemented |
| `v3.ser.03` | Round-trip with registry | User-defined types survive serialize/deserialize | ❌ Not implemented |
| `v3.ser.04` | Round-trip with ledger | Ledger history survives serialize/deserialize | ❌ Not implemented |
| `v3.ser.05` | Round-trip with stack | Stack contents survive serialize/deserialize | ❌ Not implemented |
| `v3.ser.06` | Round-trip with bridges | Registered bridges survive serialize/deserialize | ❌ Not implemented |
| `v3.ser.07` | JSON is human-readable | Serialized output uses type aliases, not raw u64 IDs | ❌ Not implemented |
| `v3.ser.08` | FFI serialize | `nexus_serialize()` returns a C string | ❌ Not implemented |
| `v3.ser.09` | FFI deserialize | `nexus_deserialize(json)` returns a valid context pointer | ❌ Not implemented |
| `v3.ser.10` | FFI free string | `nexus_free_string()` frees without crash | ❌ Not implemented |

### Assert-Gated Effects & Rollback

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v3.asr.01` | Assert consistent passes | After a Novel or Consistent apply, `assert_consistent()` → Ok | ❌ Not implemented |
| `v3.asr.02` | Assert consistent fails | After a Contradiction apply, `assert_consistent()` → Err + rollback | ❌ Not implemented |
| `v3.asr.03` | Rollback restores stack | After failed assert, stack returns to last savepoint state | ❌ Not implemented |
| `v3.asr.04` | Rollback restores ledger | After failed assert, contradictory ledger entry is removed | ❌ Not implemented |
| `v3.asr.05` | Assert type passes | `assert_type(METERS)` when top is `METERS` → Ok | ❌ Not implemented |
| `v3.asr.06` | Assert type fails | `assert_type(METERS)` when top is `SECONDS` → Err + rollback | ❌ Not implemented |
| `v3.asr.07` | Assert shape passes | `assert_shape([3])` when top has shape `[3]` → Ok | ❌ Not implemented |
| `v3.asr.08` | Nested savepoints | Pass assert (S1), compute, pass assert (S2), compute, fail assert → rollback to S2, not S1 | ❌ Not implemented |
| `v3.asr.09` | Savepoints in serialization | Serialized context includes savepoint stack | ❌ Not implemented |
| `v3.asr.10` | Manual savepoint/rollback | `savepoint()` and `rollback(id)` work without assert | ❌ Not implemented |

### AI-Native REPL

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `v3.rpl.01` | Success response format | Apply returns JSON with step, status, verdict, stack summary | 🔮 Future |
| `v3.rpl.02` | Failure response format | Failed assert returns JSON with error detail, rollback info, goal | 🔮 Future |
| `v3.rpl.03` | Syntax error on ungated effect | Side-effect without preceding assert → syntax_error response | 🔮 Future |
| `v3.rpl.04` | Goal set and echo | `goal("...")` persists across responses until `goal_done()` | 🔮 Future |
| `v3.rpl.05` | Stack summary ≤8 elements | Shows full data inline | 🔮 Future |
| `v3.rpl.06` | Stack summary >100 elements | Shows type, shape, range only | 🔮 Future |
| `v3.rpl.07` | Ledger delta reporting | Response shows what changed this step, not full history | 🔮 Future |
| `v3.rpl.08` | Legal actions included | Response lists valid next operations | 🔮 Future |
| `v3.rpl.09` | Step numbering | Each operation gets a monotonic step number | 🔮 Future |
| `v3.rpl.10` | Type names not IDs | Response uses "METERS" not "0x0002" | 🔮 Future |

---

## Python Bindings (`nexus.py`)

| ID | Test Case | Expected Behavior | Status |
|---|---|---|---|
| `py.01` | Load shared library | `nexus.py` finds and loads the `.dylib`/`.so`/`.dll` | ⚠️ Code exists, no test |
| `py.02` | Create context | `NexusContext(agent_id=1)` succeeds | ⚠️ Code exists, manual test only |
| `py.03` | Define and get types | Python-side registry mirrors Rust | ⚠️ Code exists, manual test only |
| `py.04` | Push and apply tensor | Full pipeline from Python through FFI | ⚠️ Code exists, manual test only |
| `py.05` | Apply adverb from Python | `apply_adverb(REDUCE, MULTIPLY, ...)` works | ⚠️ Code exists, manual test only |
| `py.06` | Pop result from Python | Retrieve computed values back across FFI | ❌ **FFI function missing** |
| `py.07` | Convert_to from Python | Unit conversion via Python API | ❌ **FFI function missing** |
| `py.08` | Serialize from Python | `ctx.serialize()` returns JSON string | ❌ Not implemented |
| `py.09` | Deserialize from Python | `NexusContext.deserialize(json)` returns working context | ❌ Not implemented |
| `py.10` | Context manager / cleanup | `NexusContext` properly frees Rust memory on `__del__` | ⚠️ Code exists, no test |

---

## Summary Scoreboard

**Test harness:** `nexus-core/tests/spec_tests.rs` — **89 tests, 0 failures.**

| Category | ✅ Tested | 🔧 Deferred | ❌ No Test | 🔮 Future | Total |
|---|---|---|---|---|---|
| v0.1 Registry | 8 | 0 | 0 | 0 | 8 |
| v0.1 Stack | 5 | 0 | 0 | 0 | 5 |
| v0.1 Apply | 6 | 0 | 0 | 0 | 6 |
| v0.1 Ledger | 6 | 0 | 0 | 0 | 6 |
| v0.1 Bridges | 4 | 0 | 0 | 0 | 4 |
| v0.1 FFI | 0 | 0 | 7 | 0 | 7 |
| v0.2 Tensors | 5 | 0 | 2 | 0 | 7 |
| v0.2 Structural Verbs | 0 | 0 | 7 | 0 | 7 |
| v0.2 Adverbs | 7 | 1 (Each) | 0 | 0 | 8 |
| v0.2 Stack (pick) | 7 | 0 | 0 | 0 | 7 |
| v0.3 Apple Math | 0 | 0 | 0 | 6 | 6 |
| v0.3 Serialization | 9 | 0 | 1 (FFI) | 0 | 10 |
| v0.3 Assert/Rollback | 10 | 0 | 0 | 0 | 10 |
| v0.3 Goals | 2 | 0 | 0 | 0 | 2 |
| v0.4 Provenance | 6 | 0 | 0 | 0 | 6 |
| v0.4 Ledger Queries | 3 | 0 | 0 | 0 | 3 |
| v0.4 Verified Goals | 4 | 0 | 0 | 0 | 4 |
| v0.4 Planning | 5 | 0 | 0 | 0 | 5 |
| v0.3 REPL | 0 | 0 | 0 | 10 | 10 |
| Python Bindings | 0 | 0 | 10 | 0 | 10 |
| **TOTAL** | **89** | **1** | **27** | **16** | **133** |

- ✅ = implemented and verified by a passing test
- 🔧 = deferred (Each needs monadic verbs to be meaningful)
- ❌ = no test written yet (FFI, structural verbs)
- 🔮 = future feature, not yet targeted

### Implementation History

**Session 1 (v0.1–v0.3):**
1. ~~`pick` + `drop_top`~~ ✅ Unified stack manipulation via array selection
2. ~~Context Serialization~~ ✅ Full JSON round-trip (registry, ledger, stack, bridges, goals)
3. ~~Assert-Gated Effects~~ ✅ Savepoint/rollback, assert_consistent, assert_type, assert_shape
4. ~~Scan + Table adverbs~~ ✅ Running prefix operations + outer product
5. ~~Introspection API~~ ✅ type_neighborhood, signature_count

**Session 2 (v0.4):**
6. ~~Provenance tracking~~ ✅ Every apply result carries lineage (op, input types, step number)
7. ~~Forward/backward queries~~ ✅ can_produce_from, required_for
8. ~~Verified goals~~ ✅ goal_done() returns GoalStatus::Verified or Unverified
9. ~~Goal planning~~ ✅ Declare plan upfront, track Pending→InProgress→Complete

### Remaining Priority Order

1. **FFI gaps** (7 untested) — Pop, convert, ledger verdict, serialize across FFI.
2. **Structural Verbs** (7 untested) — Shape, Reshape, Reverse, Join, Take, Drop.
3. **Each adverb** — Needs monadic verb design first.
4. **Ledger merge** — Import another context's knowledge.
5. **Recipe replay** — Re-execute a goal's operations with new inputs.
6. **Apple Math** (6 future) — Structural adverb.
7. **AI-Native REPL** (10 future) — After everything above is stable.
8. **Python Bindings** (10 untested) — After FFI is complete.

