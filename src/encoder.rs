use crate::central_bond::{CentralBondBlob, CentralEventEntry, CentralSchemaEntry};
use crate::{BondRow, BondSchema};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::collections::HashMap;

/// Supported value types for the Bond encoder
#[derive(Debug, Clone)]
pub enum BondValue<'a> {
    Float(f32),
    Int32(i32),
    String(Cow<'a, str>),
    Double(f64),
    WString(Cow<'a, str>),
    // TODO add more types as needed
}

impl<'a> BondValue<'a> {
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
            }
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
            } // Add more serialization as needed
        }
    }
}

#[derive(Debug, Clone)]
pub struct BondField<'a> {
    name: Cow<'a, str>,
    value: BondValue<'a>,
}

impl<'a> BondField<'a> {
    /// Create a new field with borrowed or owned values
    pub fn new<S>(name: S, value: BondValue<'a>) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Create field with static str and float value
    pub fn float(name: &'static str, value: f32) -> Self {
        Self {
            name: Cow::Borrowed(name),
            value: BondValue::Float(value),
        }
    }

    /// Create field with static str and int32 value
    pub fn int32(name: &'static str, value: i32) -> Self {
        Self {
            name: Cow::Borrowed(name),
            value: BondValue::Int32(value),
        }
    }

    /// Create field with static str and borrowed string value
    pub fn string(name: &'static str, value: &'a str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            value: BondValue::String(Cow::Borrowed(value)),
        }
    }

    /// Create field with static str and owned string value
    pub fn string_owned(name: &'static str, value: String) -> Self {
        Self {
            name: Cow::Borrowed(name),
            value: BondValue::String(Cow::Owned(value)),
        }
    }

    /// Create field with static str and borrowed wstring value
    pub fn wstring(name: &'static str, value: &'a str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            value: BondValue::WString(Cow::Borrowed(value)),
        }
    }

    /// Flexible method to create float field with any string type
    pub fn float_with<S>(name: S, value: f32) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            name: name.into(),
            value: BondValue::Float(value),
        }
    }

    /// Flexible method to create int32 field with any string type
    pub fn int32_with<S>(name: S, value: i32) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            name: name.into(),
            value: BondValue::Int32(value),
        }
    }

    /// Flexible method to create string field with any string type
    pub fn string_with<S, T>(name: S, value: T) -> Self
    where
        S: Into<Cow<'a, str>>,
        T: Into<Cow<'a, str>>,
    {
        Self {
            name: name.into(),
            value: BondValue::String(value.into()),
        }
    }

    /// Flexible method to create wstring field with any string type
    pub fn wstring_with<S, T>(name: S, value: T) -> Self
    where
        S: Into<Cow<'a, str>>,
        T: Into<Cow<'a, str>>,
    {
        Self {
            name: name.into(),
            value: BondValue::WString(value.into()),
        }
    }

    /// Flexible method to create double field with any string type
    pub fn double_with<S>(name: S, value: f64) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            name: name.into(),
            value: BondValue::Double(value),
        }
    }

    /// Create any field type with flexible name and value types
    pub fn new_any<S, V>(name: S, value: V) -> Self
    where
        S: Into<Cow<'a, str>>,
        V: Into<BondValue<'a>>,
    {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

// Implement From traits for common value types
impl<'a> From<i32> for BondValue<'a> {
    fn from(v: i32) -> Self {
        BondValue::Int32(v)
    }
}

impl<'a> From<f32> for BondValue<'a> {
    fn from(v: f32) -> Self {
        BondValue::Float(v)
    }
}

impl<'a> From<f64> for BondValue<'a> {
    fn from(v: f64) -> Self {
        BondValue::Double(v)
    }
}

impl<'a> From<&'a str> for BondValue<'a> {
    fn from(v: &'a str) -> Self {
        BondValue::String(Cow::Borrowed(v))
    }
}

impl<'a> From<String> for BondValue<'a> {
    fn from(v: String) -> Self {
        BondValue::String(Cow::Owned(v))
    }
}

impl<'a> From<Cow<'a, str>> for BondValue<'a> {
    fn from(v: Cow<'a, str>) -> Self {
        BondValue::String(v)
    }
}

/// Field ordering information cached per schema
#[derive(Debug)]
struct FieldOrdering {
    ordered_fields: Vec<(String, u16)>, // Field name and order ID
}

/// The main Bond encoder struct
pub struct BondEncoder {
    schema_cache: HashMap<u64, BondSchema>,
    ordering_cache: HashMap<u64, FieldOrdering>,
    row_buffer: Vec<u8>,
    tmp_buffer: Vec<u8>,
}

impl BondEncoder {
    pub fn new() -> Self {
        BondEncoder {
            schema_cache: HashMap::new(),
            ordering_cache: HashMap::new(),
            row_buffer: Vec::with_capacity(512),
            tmp_buffer: Vec::with_capacity(512),
        }
    }

    /// Create a Bond blob from a simple key-value map
    pub fn encode<'a>(
        &mut self,
        fields: &[BondField<'a>],
        event_name: &str,
        level: u8,
        metadata: &str,
    ) -> Vec<u8> {
        // 1. Create or retrieve schema from data
        let (schema_id, schema_entry) = self.create_schema_from_fields(fields);

        // 2. Create a row from the fields
        self.row_buffer.clear();

        // Get cached field ordering
        let field_ordering = self.ordering_cache.get(&schema_id).unwrap();
        // Build a map for quick field lookup by name
        let mut field_map = HashMap::with_capacity(fields.len());
        for field in fields {
            field_map.insert(field.name.as_ref(), &field.value);
        }

        // Write values in schema order
        for (field_name, _) in &field_ordering.ordered_fields {
            if let Some(value) = field_map.get(field_name.as_str()) {
                value.write_to_buffer(&mut self.row_buffer);
            }
        }

        let row_obj = BondRow::from_schema_and_row(&schema_entry.schema, &self.row_buffer);

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
    fn create_schema_from_fields<'a>(
        &mut self,
        fields: &[BondField<'a>],
    ) -> (u64, CentralSchemaEntry) {
        // Create a stable order for fields - use SmallVec to avoid allocations for small field sets
        let mut field_defs = SmallVec::<[(&str, u8, u16); 16]>::with_capacity(fields.len());

        // Prepare field definitions for schema creation (sorted by name for deterministic schema)
        for (i, field) in fields.iter().enumerate() {
            field_defs.push((
                field.name.as_ref(),
                field.value.bond_type_id(),
                (i + 1) as u16,
            ));
        }

        // Sort by name for deterministic schema creation
        field_defs.sort_by(|a, b| a.0.cmp(b.0));

        // Calculate a hash for the schema
        let schema_id = self.calculate_schema_id(&field_defs);

        // Check if we have this schema cached
        if let Some(schema) = self.schema_cache.get(&schema_id) {
            let schema_bytes = schema.as_bytes(); // Using as_bytes() which was in the original code
            let schema_md5 = self.md5_bytes(schema_bytes);

            return (
                schema_id,
                CentralSchemaEntry {
                    id: schema_id,
                    md5: schema_md5,
                    schema: schema.clone(),
                },
            );
        }

        // Create a new schema
        let schema = BondSchema::from_fields(&field_defs);

        // Cache the schema
        self.schema_cache.insert(schema_id, schema.clone());

        // Create and cache field ordering for this schema
        if !self.ordering_cache.contains_key(&schema_id) {
            let mut ordering = FieldOrdering {
                ordered_fields: Vec::with_capacity(fields.len()),
            };

            for (name, _, id) in &field_defs {
                ordering.ordered_fields.push((name.to_string(), *id));
            }

            // Sort by field ID
            ordering.ordered_fields.sort_by_key(|(_, id)| *id);
            self.ordering_cache.insert(schema_id, ordering);
        }

        // Create the schema entry
        self.tmp_buffer.clear();
        // Get the schema bytes
        let schema_bytes = schema.as_bytes(); // Using as_bytes() which was in the original code
        let schema_md5 = self.md5_bytes(schema_bytes);

        (
            schema_id,
            CentralSchemaEntry {
                id: schema_id,
                md5: schema_md5,
                schema,
            },
        )
    }

    /// Helper to calculate a schema ID based on the field definitions
    fn calculate_schema_id(&self, field_defs: &[(&str, u8, u16)]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for (name, type_id, field_id) in field_defs {
            name.hash(&mut hasher);
            type_id.hash(&mut hasher);
            field_id.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Calculate MD5 hash of data
    fn md5_bytes(&self, data: &[u8]) -> [u8; 16] {
        md5::compute(data).0
    }

    #[cfg(test)]
    pub fn schema_cache_size(&self) -> usize {
        self.schema_cache.len()
    }
}

/// Builder for creating Bond events with fluent API
pub struct BondEventBuilder<'a, 'e> {
    encoder: &'e mut BondEncoder,
    fields: Vec<BondField<'a>>,
}

impl<'a, 'e> BondEventBuilder<'a, 'e> {
    /// Create a new builder
    pub fn new(encoder: &'e mut BondEncoder) -> Self {
        Self {
            encoder,
            fields: Vec::with_capacity(16),
        }
    }

    /// Add a float field
    pub fn add_float(mut self, name: &'static str, value: f32) -> Self {
        self.fields.push(BondField::float(name, value));
        self
    }

    /// Add an int32 field
    pub fn add_int32(mut self, name: &'static str, value: i32) -> Self {
        self.fields.push(BondField::int32(name, value));
        self
    }

    /// Add a string field
    pub fn add_string(mut self, name: &'static str, value: &'a str) -> Self {
        self.fields.push(BondField::string(name, value));
        self
    }

    /// Add a string field from owned String
    pub fn add_string_owned(mut self, name: &'static str, value: String) -> Self {
        self.fields.push(BondField::string_owned(name, value));
        self
    }

    /// Add a wstring field
    pub fn add_wstring(mut self, name: &'static str, value: &'a str) -> Self {
        self.fields.push(BondField::wstring(name, value));
        self
    }

    /// Add a custom field
    pub fn add_field(mut self, field: BondField<'a>) -> Self {
        self.fields.push(field);
        self
    }

    /// Build the event and return the encoded bytes
    pub fn build(self, event_name: &str, level: u8, metadata: &str) -> Vec<u8> {
        self.encoder
            .encode(&self.fields, event_name, level, metadata)
    }
}

impl BondEncoder {
    /// Start building an event with fluent API
    pub fn builder<'a, 'e>(&'e mut self) -> BondEventBuilder<'a, 'e> {
        BondEventBuilder::new(self)
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_bond_encoder_with_fields() {
        let mut encoder = BondEncoder::new();

        let fields = [
            BondField::float("FloatCol", 3.1415),
            BondField::int32("IntCol", 42),
            BondField::string("StrCol", "hello"),
        ];

        let metadata = "namespace=testNamespace/eventVersion=Ver1v0";
        let payload = encoder.encode(&fields, "test_event", 1, metadata);

        // Basic validation that we got something
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_bond_encoder_with_builder() {
        let mut encoder = BondEncoder::new();

        let payload = encoder
            .builder()
            .add_float("FloatCol", 3.1415)
            .add_int32("IntCol", 42)
            .add_string("StrCol", "hello")
            .build(
                "test_event",
                1,
                "namespace=testNamespace/eventVersion=Ver1v0",
            );

        // Basic validation that we got something
        assert!(!payload.is_empty());
    }
    #[test]
    fn test_schema_caching() {
        let mut encoder = crate::encoder::BondEncoder::new();

        // Create fields
        let fields = [
            BondField::float("FloatCol", 3.1415),
            BondField::int32("IntCol", 42),
            BondField::string("StrCol", "hello"),
        ];

        // First encoding should create and cache the schema
        let metadata = "namespace=testNamespace/eventVersion=Ver1v0";
        let payload1 = encoder.encode(&fields, "test_event", 1, metadata);

        // Check that we have one schema in the cache
        assert_eq!(encoder.schema_cache_size(), 1);

        // Second encoding with the same fields (different values) should reuse the schema
        let fields2 = [
            BondField::float("FloatCol", 2.7182), // Different value
            BondField::int32("IntCol", 100),      // Different value
            BondField::string("StrCol", "world"), // Different value
        ];

        let payload2 = encoder.encode(&fields2, "test_event", 1, metadata);

        // Schema cache should still have just one entry
        assert_eq!(encoder.schema_cache_size(), 1);

        // Add a field to create a different schema
        let fields3 = [
            BondField::float("FloatCol", 3.1415),
            BondField::int32("IntCol", 42),
            BondField::string("StrCol", "hello"),
            BondField::int32("ExtraField", 99), // New field
        ];

        let payload3 = encoder.encode(&fields3, "test_event", 1, metadata);

        // Schema cache should now have two entries
        assert_eq!(encoder.schema_cache_size(), 2);

        // Different field order should be considered a different schema
        let fields4 = [
            BondField::int32("IntCol", 42),       // Order changed
            BondField::string("StrCol", "hello"), // Order changed
            BondField::float("FloatCol", 3.1415), // Order changed
        ];

        let payload4 = encoder.encode(&fields4, "test_event", 1, metadata);

        // Field order doesn't matter for schema ID calculation (it's based on sorted field names)
        // So we should still have just two schemas
        assert_eq!(encoder.schema_cache_size(), 2);

        // Different field types should create a new schema
        let fields5 = [
            BondField::float("FloatCol", 3.1415),
            BondField::int32("IntCol", 42),
            BondField::string("StrCol", "hello"),
            BondField::float("ExtraField", 3.14), // Same name as in fields3 but different type
        ];

        let payload5 = encoder.encode(&fields5, "test_event", 1, metadata);

        // Schema cache should now have three entries
        assert_eq!(encoder.schema_cache_size(), 3);
    }

    #[test]
    fn test_ordering_cache() {
        let mut encoder = BondEncoder::new();

        // Create fields with specific order
        let fields = [
            BondField::float("FloatCol", 3.1415),
            BondField::int32("IntCol", 42),
            BondField::string("StrCol", "hello"),
        ];

        // First encoding should create and cache the schema and ordering
        let metadata = "namespace=testNamespace";
        let payload1 = encoder.encode(&fields, "test_event", 1, metadata);

        // Change field values but use same field structure
        let fields2 = [
            BondField::float("FloatCol", 99.9),
            BondField::int32("IntCol", 123),
            BondField::string("StrCol", "world"),
        ];

        // Re-encode with same structure but different values
        let payload2 = encoder.encode(&fields2, "test_event", 1, metadata);

        // Payloads should be different due to different values
        assert_ne!(payload1, payload2);

        // But schema cache should still have just one entry
        assert_eq!(encoder.schema_cache_size(), 1);

        // Now try with fields in different order
        let fields3 = [
            BondField::string("StrCol", "hello"), // Changed order
            BondField::int32("IntCol", 42),       // Changed order
            BondField::float("FloatCol", 3.1415), // Changed order
        ];

        let payload3 = encoder.encode(&fields3, "test_event", 1, metadata);

        // Since field order is normalized in schema creation,
        // we should still have just one schema
        assert_eq!(encoder.schema_cache_size(), 1);

        // But even more importantly, the payloads should be identical
        // because the field ordering is preserved from the first encoding
        assert_eq!(payload1, payload3);
    }
}
