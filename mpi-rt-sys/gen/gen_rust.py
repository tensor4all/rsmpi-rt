#!/usr/bin/env python3
"""Generate Rust bindings for mpi-rt-sys from mpiabi definitions.

Reads mpiabi/mpi_functions.py and mpiabi/mpi_constants.py and generates:
- ../src/functions.rs  -- dynamic-dispatch function wrappers
- ../src/constants.rs  -- constant loading and accessor functions
"""

import os
import sys

# Add the gen directory to path so we can import mpiabi modules
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from mpiabi.mpi_functions import functions
from mpiabi.mpi_constants import constants

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src")

# ---- Type mapping: C type string -> Rust type string ----

TYPE_MAP = {
    "void": "()",
    "int": "c_int",
    "double": "c_double",
    "char": "c_char",
    "const void *": "*const c_void",
    "void *": "*mut c_void",
    "const char *": "*const c_char",
    "char *": "*mut c_char",
    "char **": "*mut *mut c_char",
    "char ***": "*mut *mut *mut c_char",
    "char * * *": "*mut *mut *mut c_char",
    "MPIABI_array_int_3 *": "*mut [c_int; 3]",
    "const int *": "*const c_int",
    "int *": "*mut c_int",
    "const int []": "*const c_int",
    "int []": "*mut c_int",
    "const double *": "*const c_double",
    "double *": "*mut c_double",
    "MPI_Aint": "MPI_Aint",
    "const MPI_Aint *": "*const MPI_Aint",
    "const MPI_Aint []": "*const MPI_Aint",
    "const MPI_Count []": "*const MPI_Count",
    "MPI_Aint *": "*mut MPI_Aint",
    "MPI_Count": "MPI_Count",
    "MPI_Count *": "*mut MPI_Count",
    "const MPI_Count []": "*const MPI_Count",
    "MPI_Fint": "MPI_Fint",
    "MPI_Fint *": "*mut MPI_Fint",
    "const MPI_Fint *": "*const MPI_Fint",
    "MPI_Offset": "MPI_Offset",
    "MPI_Offset *": "*mut MPI_Offset",
    "MPI_Comm": "MPI_Comm",
    "MPI_Comm *": "*mut MPI_Comm",
    "MPI_Datatype": "MPI_Datatype",
    "MPI_Datatype *": "*mut MPI_Datatype",
    "const MPI_Datatype *": "*const MPI_Datatype",
    "MPI_Datatype []": "*mut MPI_Datatype",
    "const MPI_Datatype []": "*const MPI_Datatype",
    "MPI_Errhandler": "MPI_Errhandler",
    "MPI_Errhandler *": "*mut MPI_Errhandler",
    "MPI_File": "MPI_File",
    "MPI_File *": "*mut MPI_File",
    "MPI_Group": "MPI_Group",
    "MPI_Group *": "*mut MPI_Group",
    "MPI_Info": "MPI_Info",
    "MPI_Info *": "*mut MPI_Info",
    "const MPI_Info *": "*const MPI_Info",
    "const MPI_Info []": "*const MPI_Info",
    "MPI_Message": "MPI_Message",
    "MPI_Message *": "*mut MPI_Message",
    "MPI_Op": "MPI_Op",
    "MPI_Op *": "*mut MPI_Op",
    "MPI_Request": "MPI_Request",
    "MPI_Request *": "*mut MPI_Request",
    "MPI_Status *": "*mut MPI_Status",
    "const MPI_Status *": "*const MPI_Status",
    "MPI_Win": "MPI_Win",
    "MPI_Win *": "*mut MPI_Win",
}

# Callback function pointer types
CALLBACK_TYPE_MAP = {
    "MPI_Comm_copy_attr_function *": "MPI_Comm_copy_attr_function",
    "MPI_Comm_delete_attr_function *": "MPI_Comm_delete_attr_function",
    "MPI_Comm_errhandler_function *": "MPI_Comm_errhandler_function",
    "MPI_Copy_function *": "MPI_Copy_function",
    "MPI_Datarep_conversion_function *": "MPI_Datarep_conversion_function",
    "MPI_Datarep_extent_function *": "MPI_Datarep_extent_function",
    "MPI_Delete_function *": "MPI_Delete_function",
    "MPI_File_errhandler_function *": "MPI_File_errhandler_function",
    "MPI_Grequest_cancel_function *": "MPI_Grequest_cancel_function",
    "MPI_Grequest_free_function *": "MPI_Grequest_free_function",
    "MPI_Grequest_query_function *": "MPI_Grequest_query_function",
    "MPI_Type_copy_attr_function *": "MPI_Type_copy_attr_function",
    "MPI_Type_delete_attr_function *": "MPI_Type_delete_attr_function",
    "MPI_User_function *": "MPI_User_function",
    "MPI_Win_copy_attr_function *": "MPI_Win_copy_attr_function",
    "MPI_Win_delete_attr_function *": "MPI_Win_delete_attr_function",
    "MPI_Win_errhandler_function *": "MPI_Win_errhandler_function",
}

# Constant type mapping: C type -> (Rust type, how to read from library)
CONST_TYPE_MAP = {
    "int": ("c_int", "c_int"),
    "char **": ("*mut *mut c_char", "*mut *mut c_char"),
    "char ***": ("*mut *mut *mut c_char", "*mut *mut *mut c_char"),
    "int *": ("*mut c_int", "*mut c_int"),
    "void *": ("*mut c_void", "*mut c_void"),
    "MPI_Comm": ("MPI_Comm", "MPI_Comm"),
    "MPI_Datatype": ("MPI_Datatype", "MPI_Datatype"),
    "MPI_Errhandler": ("MPI_Errhandler", "MPI_Errhandler"),
    "MPI_File": ("MPI_File", "MPI_File"),
    "MPI_Group": ("MPI_Group", "MPI_Group"),
    "MPI_Info": ("MPI_Info", "MPI_Info"),
    "MPI_Message": ("MPI_Message", "MPI_Message"),
    "MPI_Offset": ("MPI_Offset", "MPI_Offset"),
    "MPI_Op": ("MPI_Op", "MPI_Op"),
    "MPI_Request": ("MPI_Request", "MPI_Request"),
    "MPI_Status *": ("*mut MPI_Status", "*mut MPI_Status"),
    "MPI_Win": ("MPI_Win", "MPI_Win"),
    # Callback function pointers in constants
    "MPI_Comm_copy_attr_function *": ("MPI_Comm_copy_attr_function", "MPI_Comm_copy_attr_function"),
    "MPI_Comm_delete_attr_function *": ("MPI_Comm_delete_attr_function", "MPI_Comm_delete_attr_function"),
    "MPI_Copy_function *": ("MPI_Copy_function", "MPI_Copy_function"),
    "MPI_Datarep_conversion_function *": ("MPI_Datarep_conversion_function", "MPI_Datarep_conversion_function"),
    "MPI_Delete_function *": ("MPI_Delete_function", "MPI_Delete_function"),
    "MPI_Type_copy_attr_function *": ("MPI_Type_copy_attr_function", "MPI_Type_copy_attr_function"),
    "MPI_Type_delete_attr_function *": ("MPI_Type_delete_attr_function", "MPI_Type_delete_attr_function"),
    "MPI_Win_copy_attr_function *": ("MPI_Win_copy_attr_function", "MPI_Win_copy_attr_function"),
    "MPI_Win_delete_attr_function *": ("MPI_Win_delete_attr_function", "MPI_Win_delete_attr_function"),
    "MPI_Fint *": ("*mut MPI_Fint", "*mut MPI_Fint"),
}


def map_type(c_type: str) -> str:
    """Map a C type to a Rust type."""
    if c_type in TYPE_MAP:
        return TYPE_MAP[c_type]
    if c_type in CALLBACK_TYPE_MAP:
        return CALLBACK_TYPE_MAP[c_type]
    raise ValueError(f"Unknown C type: {c_type!r}")


def mpi_to_mpiabi_const(name: str) -> str:
    """Convert MPI_FOO constant name to MPIABI_FOO symbol name."""
    assert name.startswith("MPI_"), f"Expected MPI_ prefix: {name}"
    return "MPIABI_" + name[4:]


def mpi_to_rsmpi_const(name: str) -> str:
    """Convert MPI_FOO constant name to RSMPI_FOO (matching mpi-sys naming)."""
    assert name.startswith("MPI_"), f"Expected MPI_ prefix: {name}"
    return "RSMPI_" + name[4:]


def generate_functions() -> str:
    """Generate functions.rs content."""
    lines = []
    lines.append("//! MPI function bindings via dynamic loading.")
    lines.append("//!")
    lines.append("//! Auto-generated by gen_rust.py. Do not edit manually.")
    lines.append("")
    lines.append("use std::os::raw::{c_char, c_double, c_int, c_void};")
    lines.append("use std::sync::OnceLock;")
    lines.append("")
    lines.append("use crate::callback_types::*;")
    lines.append("use crate::types::*;")
    lines.append("use crate::loader;")
    lines.append("")

    for ret_type, name, params, tag in functions:
        # Build Rust parameter list
        rust_params = []
        fn_type_params = []
        call_args = []

        for p_type, p_name in params:
            rust_type = map_type(p_type)
            # Sanitize parameter name (avoid Rust keywords)
            safe_name = p_name
            if safe_name in ("type", "match", "ref", "mod", "fn", "in"):
                safe_name = safe_name + "_"
            rust_params.append(f"    {safe_name}: {rust_type},")
            fn_type_params.append(rust_type)
            call_args.append(safe_name)

        rust_ret = map_type(ret_type)
        ret_suffix = f" -> {rust_ret}" if rust_ret != "()" else ""

        fn_type_params_str = ", ".join(fn_type_params)
        fn_type_ret = f" -> {rust_ret}" if rust_ret != "()" else ""

        lines.append(f"pub unsafe fn {name}(")
        for p in rust_params:
            lines.append(p)
        lines.append(f"){ret_suffix} {{")
        lines.append(f"    type F = unsafe extern \"C\" fn({fn_type_params_str}){fn_type_ret};")
        lines.append(f"    static FN: OnceLock<F> = OnceLock::new();")
        lines.append(f"    let f = FN.get_or_init(|| {{")
        lines.append(f"        let ptr = loader::get_symbol::<F>(b\"{name}\\0\");")
        lines.append(f"        std::mem::transmute(ptr)")
        lines.append(f"    }});")
        call_args_str = ", ".join(call_args)
        lines.append(f"    f({call_args_str})")
        lines.append(f"}}")
        lines.append("")

    return "\n".join(lines)


def generate_constants() -> str:
    """Generate constants.rs content."""
    lines = []
    lines.append("//! MPI constant accessors via dynamic loading.")
    lines.append("//!")
    lines.append("//! Auto-generated by gen_rust.py. Do not edit manually.")
    lines.append("")
    lines.append("use std::os::raw::{c_char, c_double, c_int, c_void};")
    lines.append("use std::sync::OnceLock;")
    lines.append("")
    lines.append("use crate::callback_types::*;")
    lines.append("use crate::types::*;")
    lines.append("use crate::loader;")
    lines.append("")

    # Generate struct fields
    lines.append("struct MpiConstants {")
    for c_type, name in constants:
        if c_type not in CONST_TYPE_MAP:
            lines.append(f"    // SKIPPED: {name} (unknown type: {c_type})")
            continue
        rust_type, _ = CONST_TYPE_MAP[c_type]
        field_name = name.lower()
        lines.append(f"    {field_name}: {rust_type},")
    lines.append("}")
    lines.append("")
    lines.append("// SAFETY: MPI constants are initialized once and then read-only.")
    lines.append("// The raw pointers (e.g. MPI_STATUS_IGNORE) are global MPI constants")
    lines.append("// that remain valid for the lifetime of the program.")
    lines.append("unsafe impl Send for MpiConstants {}")
    lines.append("unsafe impl Sync for MpiConstants {}")
    lines.append("")

    # Generate init function
    lines.append("static CONSTANTS: OnceLock<MpiConstants> = OnceLock::new();")
    lines.append("")
    lines.append("fn get_constants() -> &'static MpiConstants {")
    lines.append("    CONSTANTS.get_or_init(|| {")
    lines.append("        let lib = loader::library();")
    lines.append("        unsafe {")
    lines.append("            MpiConstants {")
    for c_type, name in constants:
        if c_type not in CONST_TYPE_MAP:
            continue
        rust_type, _ = CONST_TYPE_MAP[c_type]
        field_name = name.lower()
        mpiabi_name = mpi_to_mpiabi_const(name)
        lines.append(f"                {field_name}: *lib.get::<{rust_type}>(b\"{mpiabi_name}\\0\").expect(\"symbol {mpiabi_name}\"),")
    lines.append("            }")
    lines.append("        }")
    lines.append("    })")
    lines.append("}")
    lines.append("")

    # Generate RSMPI_*_fn() accessor functions
    # These use the RSMPI_ naming to match what the main mpi crate expects
    for c_type, name in constants:
        if c_type not in CONST_TYPE_MAP:
            continue
        rust_type, _ = CONST_TYPE_MAP[c_type]
        rsmpi_name = mpi_to_rsmpi_const(name)
        field_name = name.lower()
        lines.append(f"pub fn {rsmpi_name}_fn() -> {rust_type} {{")
        lines.append(f"    get_constants().{field_name}")
        lines.append(f"}}")
        lines.append("")

    # Also generate MPI_* aliases (for code that uses MPI_* names directly)
    # These are extern-compatible names
    for c_type, name in constants:
        if c_type not in CONST_TYPE_MAP:
            continue
        rust_type, _ = CONST_TYPE_MAP[c_type]
        rsmpi_name = mpi_to_rsmpi_const(name)
        field_name = name.lower()
        # Also export as RSMPI_ (non-fn) for mpi-rt-sys direct use
        lines.append(f"pub fn {rsmpi_name}() -> {rust_type} {{")
        lines.append(f"    get_constants().{field_name}")
        lines.append(f"}}")
        lines.append("")

    # Additional constants that rsmpi uses but aren't in mpiabi constants
    # MAX_LIBRARY_VERSION_STRING and MAX_PROCESSOR_NAME
    lines.append("// Additional constants used by rsmpi (loaded from MPIABI symbols)")
    lines.append("pub fn RSMPI_MAX_LIBRARY_VERSION_STRING_fn() -> c_int {")
    lines.append("    // MPI standard defines this as MPI_MAX_LIBRARY_VERSION_STRING")
    lines.append("    // MPItrampoline exposes it as MPIABI_MAX_LIBRARY_VERSION_STRING")
    lines.append("    static VAL: OnceLock<c_int> = OnceLock::new();")
    lines.append("    *VAL.get_or_init(|| {")
    lines.append("        let lib = loader::library();")
    lines.append("        unsafe { *lib.get::<c_int>(b\"MPIABI_MAX_LIBRARY_VERSION_STRING\\0\").expect(\"MPIABI_MAX_LIBRARY_VERSION_STRING\") }")
    lines.append("    })")
    lines.append("}")
    lines.append("")
    lines.append("pub fn RSMPI_MAX_LIBRARY_VERSION_STRING() -> c_int {")
    lines.append("    RSMPI_MAX_LIBRARY_VERSION_STRING_fn()")
    lines.append("}")
    lines.append("")
    lines.append("pub fn RSMPI_MAX_PROCESSOR_NAME_fn() -> c_int {")
    lines.append("    static VAL: OnceLock<c_int> = OnceLock::new();")
    lines.append("    *VAL.get_or_init(|| {")
    lines.append("        let lib = loader::library();")
    lines.append("        unsafe { *lib.get::<c_int>(b\"MPIABI_MAX_PROCESSOR_NAME\\0\").expect(\"MPIABI_MAX_PROCESSOR_NAME\") }")
    lines.append("    })")
    lines.append("}")
    lines.append("")
    lines.append("pub fn RSMPI_MAX_PROCESSOR_NAME() -> c_int {")
    lines.append("    RSMPI_MAX_PROCESSOR_NAME_fn()")
    lines.append("}")
    lines.append("")

    # RSMPI_Wtime and RSMPI_Wtick (these are functions, not constants)
    lines.append("pub unsafe fn RSMPI_Wtime() -> c_double {")
    lines.append("    crate::functions::MPI_Wtime()")
    lines.append("}")
    lines.append("")
    lines.append("pub unsafe fn RSMPI_Wtick() -> c_double {")
    lines.append("    crate::functions::MPI_Wtick()")
    lines.append("}")
    lines.append("")

    # Aliases matching mpi-sys naming convention
    # mpi-sys uses RSMPI_FLOAT_COMPLEX = MPI_C_FLOAT_COMPLEX (no "C_" prefix)
    # but the auto-generated code above produces RSMPI_C_FLOAT_COMPLEX_fn (with "C_")
    lines.append("// Aliases matching mpi-sys naming convention")
    lines.append("// mpi-sys: RSMPI_FLOAT_COMPLEX = MPI_C_FLOAT_COMPLEX")
    lines.append("pub fn RSMPI_FLOAT_COMPLEX_fn() -> MPI_Datatype {")
    lines.append("    RSMPI_C_FLOAT_COMPLEX_fn()")
    lines.append("}")
    lines.append("")

    return "\n".join(lines)


def generate_callback_types() -> str:
    """Generate callback_types.rs with function pointer type aliases.

    All callback types are wrapped in Option<...> to match bindgen's convention
    for C function pointers, which are nullable by default.
    """
    lines = []
    lines.append("//! MPI callback function pointer type aliases.")
    lines.append("//!")
    lines.append("//! Auto-generated by gen_rust.py. Do not edit manually.")
    lines.append("//!")
    lines.append("//! These use `Option<fn(...)>` to match bindgen's convention for C function")
    lines.append("//! pointers, which are nullable by default.")
    lines.append("")
    lines.append("use std::os::raw::{c_int, c_void};")
    lines.append("")
    lines.append("use crate::types::*;")
    lines.append("")

    # Define callback types that appear in the MPI API
    # Each is wrapped in Option<...> to match bindgen's convention
    callbacks = {
        "MPI_User_function": "unsafe extern \"C\" fn(invec: *mut c_void, inoutvec: *mut c_void, len: *mut c_int, datatype: *mut MPI_Datatype)",
        "MPI_Comm_copy_attr_function": "unsafe extern \"C\" fn(comm: MPI_Comm, comm_keyval: c_int, extra_state: *mut c_void, attribute_val_in: *mut c_void, attribute_val_out: *mut c_void, flag: *mut c_int) -> c_int",
        "MPI_Comm_delete_attr_function": "unsafe extern \"C\" fn(comm: MPI_Comm, comm_keyval: c_int, attribute_val: *mut c_void, extra_state: *mut c_void) -> c_int",
        "MPI_Comm_errhandler_function": "unsafe extern \"C\" fn(comm: *mut MPI_Comm, error_code: *mut c_int)",
        "MPI_Win_copy_attr_function": "unsafe extern \"C\" fn(win: MPI_Win, win_keyval: c_int, extra_state: *mut c_void, attribute_val_in: *mut c_void, attribute_val_out: *mut c_void, flag: *mut c_int) -> c_int",
        "MPI_Win_delete_attr_function": "unsafe extern \"C\" fn(win: MPI_Win, win_keyval: c_int, attribute_val: *mut c_void, extra_state: *mut c_void) -> c_int",
        "MPI_Win_errhandler_function": "unsafe extern \"C\" fn(win: *mut MPI_Win, error_code: *mut c_int)",
        "MPI_Type_copy_attr_function": "unsafe extern \"C\" fn(oldtype: MPI_Datatype, type_keyval: c_int, extra_state: *mut c_void, attribute_val_in: *mut c_void, attribute_val_out: *mut c_void, flag: *mut c_int) -> c_int",
        "MPI_Type_delete_attr_function": "unsafe extern \"C\" fn(datatype: MPI_Datatype, type_keyval: c_int, attribute_val: *mut c_void, extra_state: *mut c_void) -> c_int",
        "MPI_Copy_function": "unsafe extern \"C\" fn(comm: MPI_Comm, keyval: c_int, extra_state: *mut c_void, attribute_val_in: *mut c_void, attribute_val_out: *mut c_void, flag: *mut c_int) -> c_int",
        "MPI_Delete_function": "unsafe extern \"C\" fn(comm: MPI_Comm, keyval: c_int, attribute_val: *mut c_void, extra_state: *mut c_void) -> c_int",
        "MPI_File_errhandler_function": "unsafe extern \"C\" fn(file: *mut MPI_File, error_code: *mut c_int)",
        "MPI_Grequest_query_function": "unsafe extern \"C\" fn(extra_state: *mut c_void, status: *mut MPI_Status) -> c_int",
        "MPI_Grequest_free_function": "unsafe extern \"C\" fn(extra_state: *mut c_void) -> c_int",
        "MPI_Grequest_cancel_function": "unsafe extern \"C\" fn(extra_state: *mut c_void, complete: c_int) -> c_int",
        "MPI_Datarep_conversion_function": "unsafe extern \"C\" fn(userbuf: *mut c_void, datatype: MPI_Datatype, count: c_int, filebuf: *mut c_void, position: MPI_Offset, extra_state: *mut c_void) -> c_int",
        "MPI_Datarep_extent_function": "unsafe extern \"C\" fn(datatype: MPI_Datatype, extent: *mut MPI_Aint, extra_state: *mut c_void) -> c_int",
    }

    for name, sig in callbacks.items():
        lines.append(f"pub type {name} = Option<")
        lines.append(f"    {sig},")
        lines.append(f">;")
        lines.append("")

    return "\n".join(lines)


def main():
    os.makedirs(OUT_DIR, exist_ok=True)

    # Generate functions.rs
    functions_content = generate_functions()
    path = os.path.join(OUT_DIR, "functions.rs")
    with open(path, "w") as f:
        f.write(functions_content)
    print(f"Generated {path} ({len(functions)} functions)")

    # Generate constants.rs
    constants_content = generate_constants()
    path = os.path.join(OUT_DIR, "constants.rs")
    with open(path, "w") as f:
        f.write(constants_content)
    print(f"Generated {path} ({len(constants)} constants)")

    # Generate callback_types.rs
    callback_content = generate_callback_types()
    path = os.path.join(OUT_DIR, "callback_types.rs")
    with open(path, "w") as f:
        f.write(callback_content)
    print(f"Generated {path}")


if __name__ == "__main__":
    main()
