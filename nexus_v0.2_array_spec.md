# NEXUS
## Array Semantics Extension
**Product Specification v0.2**

**Status:** Draft — Extension  
**Target:** Augmenting Nexus with Array Programming Language Paradigms (APL, Uiua, BQN)

---

## 1. The Intersection of Arrays and Semantics
Nexus v0.1 established a stack-based virtual machine where every value has an attached semantic identifier (an "Ontic Type"). 

However, in real-world AI contexts (like neural network tensors or geometric datasets), we rarely operate on single floats. We operate on collections. Array languages like APL, J, BQN, and Uiua excel at manipulating collections tersely and powerfully without explicit loops.

Nexus v0.2 evolves the stack so that **every item is a typed, multi-dimensional array (Tensor)**. By merging Array and Stack paradigms (heavily inspired by [Uiua](https://www.uiua.org/)), we unlock powerful data manipulation that strictly preserves semantic meaning.

### 1.1 Tensors over Scalars
A scalar in Nexus v0.2 is merely a tensor of shape `[]`. A list is a tensor of shape `[N]`. A matrix is `[N, M]`.

```rust
// Old TypedValue (v0.1)
struct TypedValue { value: f64, ontic_type: u64 }

// New TypedValue (v0.2)
struct TypedTensor { 
    data: Vec<f64>, 
    shape: Vec<usize>, 
    ontic_type: u64 
}
```

---

## 2. Built-in Verbs (Core Operations)
Verbs are operations that consume 1 or 2 tensors from the stack and produce a new tensor. They are natively **Rank Polymorphic**, meaning they automatically broadcast across shapes.

### 2.1 Pervasive Math Verbs
Math verbs map across arrays element-wise.
*   **Add (+)**
*   **Subtract (-)**
*   **Multiply (×)**
*   **Divide (÷)**
*   **Max (⌈)**
*   **Min (⌊)**

*Semantic Rule:* When applying `Add` to two vectors `[1, 2, 3] :: METERS` and `[4, 5, 6] :: METERS`, the result is `[5, 7, 9] :: METERS`. The semantic rules from v0.1 still govern the `ontic_type`, applying globally to the entire tensor.

### 2.2 Structural Verbs
Structural verbs alter the shape or order of the tensor, but generally **preserve the semantic type**.
*   **Shape (△):** Pushes the shape of the tensor. Outputs an untyped (or `SCALAR`) array.
*   **Reshape (↯):** Takes a shape tensor and a data tensor. Reshapes the data.
*   **Reverse (⇌):** Reverses the array along the first axis.
*   **Join (⊂):** Concatenates two arrays. They must have the same `ontic_type`.
*   **Take (↙)** / **Drop (↘):** Slices the array.

---

## 3. Built-in Adverbs (Modifiers)
Adverbs are higher-order functions. They take a Verb and modify its behavior across an array. This is where the power of array languages truly shines.

### 3.1 Reduce (/)
`Reduce` folds an array using a dyadic verb.
*Example:* `Reduce(Add)` on `[1, 2, 3, 4] :: METERS` produces `10 :: METERS`.

**Semantic Challenge:** 
If we `Reduce(Multiply)` on `[2, 2, 2] :: METERS`, what is the type? 
Step 1: `METER × METER = AREA`
Step 2: `AREA × METER = VOLUME`
Because Nexus is a ledger of nominal declarations, the user must explicitly declare the output type of the reduction (`VOLUME`). The Ledger records the consistency of the entire reduction chain!

### 3.2 Each (¨)
`Each` applies a verb or block to every element of a tensor individually.
Useful for lifting operations that expect scalars into array operations.

### 3.3 Scan (\)
`Scan` computes the cumulative prefix applying a verb.
*Example:* `Scan(Add)` on `[1, 2, 3] :: SECONDS` yields `[1, 3, 6] :: SECONDS`.

### 3.4 Table (⊞)
`Table` (Outer Product) applies a verb between all pairs of elements from two arrays.
*Example:* `Table(Multiply)` between `[2, 3] :: METERS` and `[4, 5] :: METERS` yields a `2x2` matrix `[[8, 10], [12, 15]] :: AREA`.

---

## 4. Stack Topology and Data Flow

Stack-based array languages compose beautifully. Let's trace calculating the semantic geometric mean of two lengths:

1. Push `[4.0, 9.0]` typed as `METERS`
2. Push `[2.0]` (the count) typed as `SCALAR`
3. Apply `Reduce(Multiply)` -> Outputs `36.0` typed as `AREA`
4. Apply `Root` (taking `36.0 :: AREA` and `2.0 :: SCALAR`) -> Outputs `6.0 :: METERS`

```python
# With Python Nexus API
ctx.push_array([4.0, 9.0], shape=[2], ontic_type=meters)
ctx.apply_adverb(Adverb.REDUCE, Op.MULTIPLY, expected_output_type=area)

ctx.push_scalar(2.0, scalar)
ctx.apply(Op.ROOT, [area, scalar], [meters])

print(ctx.pop()) # Tensor: [6.0], Shape: [], Type: METERS
```

---

## 5. Implementation Path

To upgrade `nexus-core` from v0.1 to v0.2:
1. Modify `TypedValue` to contain a `Vec<f64>` and a `Vec<usize>` (shape).
2. Implement NumPy-style broadcasting for `Op::Add`, `Subtract`, `Multiply`, `Divide`.
3. Add a new `Adverb` enum and an `apply_adverb` method to `NexusContext`.
4. Update the FFI to accept `const double*` arrays and `const usize*` shape dimensions.
