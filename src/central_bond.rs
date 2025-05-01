//use md5;

use crate::{BondRow as FfiBondRow, BondSchema as FfiBondSchema};

/// Helper to encode UTF-8 Rust str to UTF-16LE bytes
fn utf8_to_utf16le_bytes(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
}

/// Helper to calculate MD5 hash, returns [u8;16]
//fn md5_bytes(data: &[u8]) -> [u8; 16] {
//    md5::compute(data).0
//}

/// Schema entry for central blob
pub struct CentralSchemaEntry {
    pub id: u64,
    pub md5: [u8; 16],
    pub schema: FfiBondSchema,
}

/// Event/row entry for central blob
pub struct CentralEventEntry {
    pub schema_id: u64,
    pub level: u8,
    pub event_name: String,
    pub row: FfiBondRow,
}

pub struct CentralBondBlob {
    pub version: u32,
    pub format: u32,
    pub metadata: String, // UTF-8, will be stored as UTF-16LE
    pub schemas: Vec<CentralSchemaEntry>,
    pub events: Vec<CentralEventEntry>,
}

const TERMINATOR: u64 = 0xdeadc0dedeadc0de;

impl CentralBondBlob {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // HEADER
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.format.to_le_bytes());

        // METADATA (len, UTF-16LE bytes)
        let metadata_utf16 = utf8_to_utf16le_bytes(&self.metadata);
        buf.extend_from_slice(&(metadata_utf16.len() as u32).to_le_bytes());
        buf.extend_from_slice(&metadata_utf16);

        // SCHEMAS (type 0)
        for schema in &self.schemas {
            buf.extend_from_slice(&0u16.to_le_bytes()); // entity type 0
            buf.extend_from_slice(&schema.id.to_le_bytes());
            buf.extend_from_slice(&schema.md5);
            let schema_bytes = schema.schema.as_bytes();
            buf.extend_from_slice(&(schema_bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(schema_bytes);
            buf.extend_from_slice(&TERMINATOR.to_le_bytes());
        }

        // EVENTS (type 2)
        for event in &self.events {
            buf.extend_from_slice(&2u16.to_le_bytes()); // entity type 2
            buf.extend_from_slice(&event.schema_id.to_le_bytes());
            buf.push(event.level);

            // event name (UTF-16LE, prefixed with u16 len in bytes)
            let evname_utf16 = utf8_to_utf16le_bytes(&event.event_name);
            buf.extend_from_slice(&(evname_utf16.len() as u16).to_le_bytes());
            buf.extend_from_slice(&evname_utf16);

            // MODIFIED: Add the Simple Protocol header before the row data
            let row_bytes = event.row.as_bytes();

            // Create a new buffer with the SP header
            let mut modified_row = Vec::with_capacity(row_bytes.len() + 4);
            modified_row.extend_from_slice(&[0x53, 0x50, 0x01, 0x00]); // Simple Protocol header
            modified_row.extend_from_slice(row_bytes);

            // row (len, bytes)
            //let row_bytes = event.row.as_bytes();
            //buf.extend_from_slice(&(row_bytes.len() as u32).to_le_bytes());
            //buf.extend_from_slice(row_bytes);

            // Write the length followed by the modified row
            buf.extend_from_slice(&(modified_row.len() as u32).to_le_bytes());
            buf.extend_from_slice(&modified_row);

            buf.extend_from_slice(&TERMINATOR.to_le_bytes());
        }

        buf
    }
}

// Example usage/test (can be moved to examples or tests)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BondRow, BondSchema};
    use md5;

    //Helper to calculate MD5 hash, returns [u8;16]
    fn md5_bytes(data: &[u8]) -> [u8; 16] {
        md5::compute(data).0
    }

    #[test]
    fn test_central_bond_blob_creation() {
        // Prepare a schema
        let fields = &[
            ("foo", 16u8, 1u16), // BT_INT32
            ("bar", 9u8, 2u16),  // BT_STRING
        ];
        let schema_obj = BondSchema::from_fields(fields);
        let schema_bytes = schema_obj.as_bytes().to_vec();
        let schema_md5 = md5_bytes(&schema_bytes);
        let schema_id = 1234u64;

        let schema = CentralSchemaEntry {
            id: schema_id,
            md5: schema_md5,
            schema: schema_obj,
        };

        // Prepare a row
        let mut row = Vec::new();
        row.extend_from_slice(&42i32.to_le_bytes());
        let s = "hello";
        row.extend_from_slice(&(s.len() as u32).to_le_bytes()); // Bond expects u32 LE for string length
        row.extend_from_slice(s.as_bytes());

        let row_obj = BondRow::from_schema_and_row(&schema.schema, &row);

        let event = CentralEventEntry {
            schema_id,
            level: 0, // e.g. ETW verbose
            event_name: "eventname".to_string(),
            row: row_obj,
        };

        // Metadata
        let metadata =
            "namespace=testNamespace/eventVersion=Ver1v0/tenant=T/role=R/roleinstance=RI";

        // Build blob
        let blob = CentralBondBlob {
            version: 1,
            format: 42,
            metadata: metadata.to_string(),
            schemas: vec![schema],
            events: vec![event],
        };

        let payload = blob.to_bytes();

        // Only assert that the payload is created and non-empty
        assert!(!payload.is_empty());
    }
}
