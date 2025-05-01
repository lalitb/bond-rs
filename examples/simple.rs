use bond_rs::{BondRow, BondSchema};

fn main() {
    // Example schema: FloatCol (double, id=1), Level (int32, id=2)
    let schema = BondSchema::from_fields(&[("FloatCol", 8, 1), ("Level", 2, 2)]);

    // Build the row: FloatCol = 3.1415, Level = 4
    let mut row = Vec::new();
    row.extend_from_slice(&3.1415f64.to_le_bytes());
    row.extend_from_slice(&4i32.to_le_bytes());

    let bond_row = BondRow::from_schema_and_row(&schema, &row);

    // Use the schema and row Bond blobs as needed (e.g., send to another service, write to disk, etc.)
    println!(
        "Bond schema blob ({} bytes): {:x?}",
        schema.as_bytes().len(),
        schema.as_bytes()
    );
    println!(
        "Bond row blob ({} bytes): {:x?}",
        bond_row.as_bytes().len(),
        bond_row.as_bytes()
    );
}
