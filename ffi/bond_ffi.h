#pragma once
#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

struct BondSchemaResult {
    void* schema_ptr;     // Opaque pointer to SchemaDef
    void* schema_bytes;       // Marshaled bytes (for Rust)
    size_t schema_bytes_len;
};

// Marshals a schema buffer describing the fields into a Bond-encoded schema blob and a schema pointer.
// schema_buf: pointer to binary schema description
// schema_len: length of schema_buf
// out_len: output parameter to receive size of returned buffer
// Returns: malloc-allocated pointer to BondSchemaResult (must be freed via bond_ffi_free_schema_result)
BondSchemaResult* bond_ffi_marshal_schema(const void* schema_buf, size_t schema_len, size_t* out_len);

// Marshals a data row using a Bond schema pointer and a binary row buffer.
// schema_ptr: pointer to the Bond SchemaDef (from BondSchemaResult->schema_ptr)
// row_buf: pointer to binary row data (field values in schema order)
// row_len: length of row_buf
// out_len: output parameter to receive size of returned buffer
// Returns: malloc-allocated pointer to Bond row blob (must be freed via bond_ffi_free)
void* bond_ffi_marshal_row(void* schema_ptr,
                           const void* row_buf, size_t row_len,
                           size_t* out_len);

// Frees a buffer allocated by the above functions.
void bond_ffi_free(void* ptr);

// Frees a BondSchemaResult structure and its contents.
void bond_ffi_free_schema_result(BondSchemaResult* result);


#ifdef __cplusplus
}
#endif