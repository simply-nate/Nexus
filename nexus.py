import ctypes
import os
import sys

# Load the shared library
lib_name = "libnexus_core.so"
if sys.platform == "darwin":
    lib_name = "libnexus_core.dylib"
elif sys.platform == "win32":
    lib_name = "nexus_core.dll"

# Try to find the library in the target/release directory first
lib_path = os.path.join(os.path.dirname(__file__), "nexus-core", "target", "release", lib_name)
if not os.path.exists(lib_path):
    lib_path = os.path.join(os.path.dirname(__file__), lib_name) # Fallback

if not os.path.exists(lib_path):
    raise RuntimeError(f"Could not find Nexus shared library: {lib_path}. Did you run 'cargo build --release'?")

nexus_lib = ctypes.CDLL(lib_path)

# ==========================================
# C-FFI Signature Definitions
# ==========================================
# pub extern "C" fn nexus_context_new(agent_id: u64) -> *mut NexusContext;
nexus_lib.nexus_context_new.argtypes = [ctypes.c_uint64]
nexus_lib.nexus_context_new.restype = ctypes.c_void_p

# pub extern "C" fn nexus_push_f64(ctx: *mut NexusContext, value: f64, ontic_type: u64) -> i32;
nexus_lib.nexus_push_f64.argtypes = [ctypes.c_void_p, ctypes.c_double, ctypes.c_uint64]
nexus_lib.nexus_push_f64.restype = ctypes.c_int32

# pub extern "C" fn nexus_apply(ctx: *mut NexusContext, op: u32, input_types: *const u64, input_len: usize, output_types: *const u64, output_len: usize) -> i32;
nexus_lib.nexus_apply.argtypes = [
    ctypes.c_void_p, 
    ctypes.c_uint32, 
    ctypes.POINTER(ctypes.c_uint64), 
    ctypes.c_size_t, 
    ctypes.POINTER(ctypes.c_uint64), 
    ctypes.c_size_t
]
nexus_lib.nexus_apply.restype = ctypes.c_int32

# pub extern "C" fn nexus_context_free(ctx: *mut NexusContext);
nexus_lib.nexus_context_free.argtypes = [ctypes.c_void_p]
nexus_lib.nexus_context_free.restype = None


# ==========================================
# Python Wrapper API
# ==========================================
class Op:
    ADD = 1
    SUBTRACT = 2
    MULTIPLY = 3
    DIVIDE = 4

# Pre-defined layer 0 IDs
NULL_TYPE = 0x0000
SCALAR    = 0x0001
METER     = 0x0002
KILOGRAM  = 0x0003
SECOND    = 0x0004
BIT       = 0x0005

class NexusContext:
    def __init__(self, agent_id: int = 1):
        self._ctx = nexus_lib.nexus_context_new(agent_id)
        if not self._ctx:
            raise RuntimeError("Failed to create NexusContext")
        
        # Local registry mockup for Python side (until FFI handles registry full exposure)
        self._next_auto_id = 0x1000
        self._registry = {
            "NULL_TYPE": NULL_TYPE,
            "SCALAR": SCALAR,
            "METER": METER,
            "KILOGRAM": KILOGRAM,
            "SECOND": SECOND,
            "BIT": BIT,
        }

    def define(self, name: str) -> int:
        if name in self._registry:
            raise ValueError(f"Type already defined: {name}")
        id_val = self._next_auto_id
        self._next_auto_id += 1
        self._registry[name] = id_val
        return id_val

    def define_explicit(self, name: str, id_val: int):
        if name in self._registry:
            raise ValueError(f"Type already defined: {name}")
        if id_val >= self._next_auto_id:
            self._next_auto_id = id_val + 1
        self._registry[name] = id_val

    def get(self, name: str) -> int:
        if name not in self._registry:
            raise ValueError(f"Type not found: {name}")
        return self._registry[name]

    def push(self, value: float, ontic_type: int):
        result = nexus_lib.nexus_push_f64(self._ctx, float(value), ontic_type)
        if result != 0:
            raise RuntimeError(f"Error pushing value (code {result})")

    def apply(self, op: int, input_types: list[int], output_types: list[int]):
        in_array = (ctypes.c_uint64 * len(input_types))(*input_types)
        out_array = (ctypes.c_uint64 * len(output_types))(*output_types)
        
        result = nexus_lib.nexus_apply(
            self._ctx, 
            op, 
            in_array, len(input_types), 
            out_array, len(output_types)
        )
        if result != 0:
            raise RuntimeError(f"Apply failed. Verdict: Contradiction or Error (code {result})")
        print(f"Apply OP={op} successful! Output types: {output_types}")

    def __del__(self):
        if hasattr(self, '_ctx') and self._ctx:
            nexus_lib.nexus_context_free(self._ctx)
            self._ctx = None

if __name__ == "__main__":
    print("Nexus Python bindings initialized.")
    ctx = NexusContext(agent_id=101)
    
    # Example usage
    meters = ctx.get("METER")
    area = ctx.define("AREA")
    
    ctx.push(3.0, meters)
    ctx.push(4.0, meters)
    
    try:
        ctx.apply(Op.MULTIPLY, [meters, meters], [area])
        print("Test passed: 3 METERS * 4 METERS = 12 AREA")
    except Exception as e:
        print(f"Test failed: {e}")
