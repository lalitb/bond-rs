use std::os::raw::c_void;

extern "C" {
    pub fn bond_ffi_marshal_schema(schema_buf: *const c_void, schema_len: usize, out_len: *mut usize) -> *mut c_void;
    pub fn bond_ffi_marshal_row(schema_bytes: *const c_void, schema_len: usize, row_buf: *const c_void, row_len: usize, out_len: *mut usize) -> *mut c_void;
    pub fn bond_ffi_free(ptr: *mut c_void);
} 