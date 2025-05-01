#pragma once
#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

// Marshals a schema buffer describing the fields into a Bond-encoded schema blob.
// schema_buf: pointer to binary schema description
// schema_len: length of schema_buf
// out_len: output parameter to receive size of returned buffer
// Returns: malloc-allocated pointer to Bond schema blob (must be freed via bond_ffi_free)
void* bond_ffi_marshal_schema(const void* schema_buf, size_t schema_len, size_t* out_len);

// Marshals a data row using a Bond schema blob and a binary row buffer.
// schema_bytes: pointer to the Bond schema blob
// schema_len: length of schema_blob
// row_buf: pointer to binary row data (field values in schema order)
// row_len: length of row_buf
// out_len: output parameter to receive size of returned buffer
// Returns: malloc-allocated pointer to Bond row blob (must be freed via bond_ffi_free)
void* bond_ffi_marshal_row(const void* schema_bytes, size_t schema_len,
                           const void* row_buf, size_t row_len,
                           size_t* out_len);

// Frees a buffer allocated by the above functions.
void bond_ffi_free(void* ptr);

#ifdef __cplusplus
}
#endif