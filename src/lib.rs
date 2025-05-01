pub mod central_bond;
pub mod encoder;
mod ffi;

use std::slice;

pub struct BondSchema {
    bytes: Vec<u8>,
    fields: Vec<(String, u8, u16)>, // (name, type, id)
}

pub struct BondRow {
    bytes: Vec<u8>,
}

impl Clone for BondSchema {
    fn clone(&self) -> Self {
        BondSchema {
            bytes: self.bytes.clone(),
            fields: self.fields.clone(),
        }
    }
}

impl BondSchema {
    pub fn from_fields(fields: &[(&str, u8, u16)]) -> Self {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(fields.len() as u16).to_le_bytes());
        for (name, typ, id) in fields {
            buf.push(name.len() as u8);
            buf.extend_from_slice(name.as_bytes());
            buf.push(*typ);
            buf.extend_from_slice(&id.to_le_bytes());
        }
        let mut out_len = 0usize;
        let ptr = unsafe {
            ffi::bond_ffi_marshal_schema(buf.as_ptr() as *const _, buf.len(), &mut out_len)
        };
        assert!(!ptr.is_null());
        let bytes = unsafe {
            let slice = slice::from_raw_parts(ptr as *const u8, out_len);
            let v = slice.to_vec();
            ffi::bond_ffi_free(ptr);
            v
        };
        let fields = fields
            .iter()
            .map(|(name, typ, id)| (name.to_string(), *typ, *id))
            .collect();
        BondSchema { bytes, fields }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl BondRow {
    pub fn from_schema_and_row(schema: &BondSchema, row: &[u8]) -> Self {
        let mut out_len = 0usize;
        let ptr = unsafe {
            ffi::bond_ffi_marshal_row(
                schema.bytes.as_ptr() as *const _,
                schema.bytes.len(),
                row.as_ptr() as *const _,
                row.len(),
                &mut out_len,
            )
        };
        assert!(!ptr.is_null());
        let bytes = unsafe {
            let slice = slice::from_raw_parts(ptr as *const u8, out_len);
            let v = slice.to_vec();
            ffi::bond_ffi_free(ptr);
            v
        };
        BondRow { bytes }
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_schema_from_fields() {
        let fields = &[
            ("foo", 16u8, 1u16), // BT_INT32 = 16
            ("bar", 9u8, 2u16),  // BT_STRING = 9
        ];
        let schema = BondSchema::from_fields(fields);
        // Should not be empty
        assert!(!schema.bytes.is_empty());
        // Should be accessible
        assert!(!schema.as_bytes().is_empty());
    }

    #[test]
    fn test_bond_row_from_schema_and_row() {
        let fields = &[
            ("foo", 16u8, 1u16), // BT_INT32 = 16
            ("bar", 9u8, 2u16),  // BT_STRING = 9
        ];
        let schema = BondSchema::from_fields(fields);

        // Compose a row: foo = 42i32; bar = "hello"
        let mut row = Vec::new();
        row.extend_from_slice(&42i32.to_le_bytes()); // foo
        let s = "hello";
        row.extend_from_slice(&(s.len() as u16).to_le_bytes()); // bar string length (u16 LE)
        row.extend_from_slice(s.as_bytes()); // bar string bytes

        let bond_row = BondRow::from_schema_and_row(&schema, &row);
        assert!(!bond_row.bytes.is_empty());
        assert!(!bond_row.as_bytes().is_empty());
    }
}
