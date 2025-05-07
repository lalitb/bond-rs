use std::os::raw::c_void;

#[repr(C)]
pub struct BondSchemaResult {
    pub schema_ptr: *mut c_void,
    pub schema_bytes: *mut c_void,
    pub schema_bytes_len: usize,
}

extern "C" {
    pub fn bond_ffi_marshal_schema(
        schema_buf: *const c_void,
        schema_len: usize,
        out_len: *mut usize,
    ) -> *mut BondSchemaResult;

    pub fn bond_ffi_marshal_row(
        schema_ptr: *mut c_void, // NOTE: This is the schema_ptr from BondSchemaResult!
        row_buf: *const c_void,
        row_len: usize,
        out_len: *mut usize,
    ) -> *mut c_void;

    pub fn bond_ffi_free(ptr: *mut c_void);

    pub fn bond_ffi_free_schema_result(ptr: *mut BondSchemaResult);
}
