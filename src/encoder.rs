use std::collections::HashMap;
use crate::{BondSchema, BondRow};
use crate::central_bond::{CentralBondBlob, CentralSchemaEntry, CentralEventEntry};

/// Supported value types for the Bond encoder
#[derive(Debug, Clone)]
pub enum BondValue {
    Float(f32),
    Int32(i32),
    String(String),
    Double(f64),
    WString(String), // UTF-16LE encoded
    // Add more types as needed
}

impl BondValue {
    /// Get the Bond type ID for this value
    /// These values map to specific data types: BT_BOOL(2), BT_FLOAT(7), BT_DOUBLE(8), 
    /// BT_STRING(9), BT_WSTRING(18), BT_INT32(16), BT_INT64(17)
    fn bond_type_id(&self) -> u8 {
        match self {
            BondValue::Float(_) => 7,    // BT_DOUBLE
            BondValue::Double(_) => 8,   // BT_DOUBLE
            BondValue::Int32(_) => 16,   // BT_INT32
            BondValue::String(_) => 9,   // BT_STRING
            BondValue::WString(_) => 18, // BT_WSTRING
            // Add more mappings as needed
        }
    }
    
    /// Write the value bytes to a buffer
    fn write_to_buffer(&self, buffer: &mut Vec<u8>) {
        match self {
            BondValue::Float(v) => buffer.extend_from_slice(&v.to_le_bytes()),
            BondValue::Int32(v) => buffer.extend_from_slice(&v.to_le_bytes()),
            BondValue::String(v) => {
                buffer.extend_from_slice(&(v.len() as u16).to_le_bytes());
                buffer.extend_from_slice(v.as_bytes());
            },
            BondValue::Double(v) => buffer.extend_from_slice(&v.to_le_bytes()),
            BondValue::WString(v) => {
                // Convert UTF-8 to UTF-16
                let utf16: Vec<u16> = v.encode_utf16().collect();
                
                // Write length of UTF-16 string (in code units, not bytes)
                buffer.extend_from_slice(&(utf16.len() as u16).to_le_bytes());
                
                // Write UTF-16LE bytes
                for code_unit in utf16 {
                    buffer.extend_from_slice(&code_unit.to_le_bytes());
                }
            },
            // Add more serialization as needed
        }
    }
}

/// The main Bond encoder struct
pub struct BondEncoder {
    schema_cache: HashMap<u64, BondSchema>,
}

impl BondEncoder {
    pub fn new() -> Self {
        BondEncoder {
            schema_cache: HashMap::new(),
        }
    }
    
    /// Create a Bond blob from a simple key-value map
    pub fn encode(&mut self, 
                  data: &HashMap<String, BondValue>, 
                  event_name: &str, 
                  level: u8,
                  metadata: &str) -> Vec<u8> {
        
        // 1. Create or retrieve schema from data
        let (schema_id, schema_entry) = self.create_schema_from_data(data);
        
        // 2. Create a row from the data
        let row_bytes = self.create_row_bytes(data, &schema_entry);
        let row_obj = BondRow::from_schema_and_row(&schema_entry.schema, &row_bytes);
        
        // 3. Create event entry
        let event = CentralEventEntry {
            schema_id,
            level,
            event_name: event_name.to_string(),
            row: row_obj,
        };
        
        // 4. Create the bond blob
        let blob = CentralBondBlob {
            version: 1,
            format: 2,
            metadata: metadata.to_string(),
            schemas: vec![schema_entry],
            events: vec![event],
        };
        
        // 5. Return the serialized blob
        blob.to_bytes()
    }
    
    /// Create or retrieve a schema for the given data
    fn create_schema_from_data(&mut self, data: &HashMap<String, BondValue>) -> (u64, CentralSchemaEntry) {
        // Create a stable order for fields
        let mut fields = Vec::with_capacity(data.len());
        
        // Sort keys for deterministic schema creation
        let mut keys: Vec<&String> = data.keys().collect();
        keys.sort();
        
        // Build field definitions in sorted order
        for (i, key) in keys.iter().enumerate() {
            let value = &data[*key];
            fields.push((key.as_str(), value.bond_type_id(), (i + 1) as u16));
        }
        
        // Calculate a hash for the schema
        let schema_id = self.calculate_schema_id(&fields);
        
        // Check if we have this schema cached
        if let Some(schema) = self.schema_cache.get(&schema_id) {
            let schema_bytes = schema.as_bytes();
            let schema_md5 = self.md5_bytes(schema_bytes);
            
            return (schema_id, CentralSchemaEntry {
                id: schema_id,
                md5: schema_md5,
                schema: schema.clone(),
            });
        }
        
        // Create a new schema
        let schema = BondSchema::from_fields(&fields);
        
        // Cache the schema
        self.schema_cache.insert(schema_id, schema.clone());
        
        // Create the schema entry
        let schema_bytes = schema.as_bytes();
        let schema_md5 = self.md5_bytes(schema_bytes);
        
        (schema_id, CentralSchemaEntry {
            id: schema_id,
            md5: schema_md5,
            schema,
        })
    }
    
    /// Helper to calculate a schema ID based on the field definitions
    fn calculate_schema_id(&self, fields: &[(&str, u8, u16)]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for (name, type_id, field_id) in fields {
            name.hash(&mut hasher);
            type_id.hash(&mut hasher);
            field_id.hash(&mut hasher);
        }
        
        hasher.finish()
    }
    
    /// Create row bytes from data
    fn create_row_bytes(&self, data: &HashMap<String, BondValue>, schema_entry: &CentralSchemaEntry) -> Vec<u8> {
        // Build a map of field names to their order for quick lookup
        let field_order = self.extract_field_ordering_from_schema(&schema_entry.schema);
        
        // Pre-allocate a reasonable buffer size
        let mut buffer = Vec::with_capacity(data.len() * 8);
        
        // Sort field names by their order
        let mut ordered_fields: Vec<(&String, &u16)> = field_order.iter().collect();
        ordered_fields.sort_by_key(|x| *x.1);
        
        // Write values in schema order
        for (field_name, _) in ordered_fields {
            if let Some(value) = data.get(field_name) {
                value.write_to_buffer(&mut buffer);
            } else {
                // Skip missing fields
                // Note: This assumes fields are optional. If required, handle appropriately.
            }
        }
        
        buffer
    }
    
    /// Extract field ordering from schema
    /// Returns a map of field name to field order
    fn extract_field_ordering_from_schema(&self, schema: &BondSchema) -> HashMap<String, u16> {
        let mut field_order = HashMap::new();
        for (name, _, id) in &schema.fields {
            field_order.insert(name.clone(), *id);
        }
        field_order
    }
    
    /// Calculate MD5 hash of data
    fn md5_bytes(&self, data: &[u8]) -> [u8; 16] {
        md5::compute(data).0
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bond_encoder() {
        let mut encoder = BondEncoder::new();
        
        let mut data = HashMap::new();
        data.insert("FloatCol".to_string(), BondValue::Float(3.1415));
        data.insert("IntCol".to_string(), BondValue::Int32(42));
        data.insert("StrCol".to_string(), BondValue::String("hello".to_string()));
        
        let metadata = "namespace=testNamespace/eventVersion=Ver1v0";
        let payload = encoder.encode(&data, "test_event", 1, metadata);
        
        // Basic validation that we got something
        assert!(!payload.is_empty());
    }
}

