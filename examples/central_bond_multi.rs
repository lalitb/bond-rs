use bond_rs::{BondSchema, BondRow};
use bond_rs::central_bond::{CentralBondBlob, CentralSchemaEntry, CentralEventEntry};
use std::fs::File;
use std::io::Write;

// Helper for MD5
fn md5_bytes(data: &[u8]) -> [u8; 16] {
    md5::compute(data).0
}

// LZ4 chunked compressor (64 KiB chunks)
fn lz4_chunked_compress(input: &[u8]) -> Vec<u8> {
    use lz4_flex::block::compress_into;
    const CHUNK_SIZE: usize = 64 * 1024;
    let mut output = Vec::new();
    let mut offset = 0;
    while offset < input.len() {
        let end = usize::min(offset + CHUNK_SIZE, input.len());
        let chunk = &input[offset..end];
        let max_compressed_size = lz4_flex::block::get_maximum_output_size(chunk.len());
        let mut compressed = vec![0u8; max_compressed_size];
        let compressed_size = compress_into(chunk, &mut compressed).expect("Compression failed");
        let len_bytes = (compressed_size as u32).to_le_bytes();
        output.extend_from_slice(&len_bytes);
        output.extend_from_slice(&compressed[..compressed_size]);
        offset = end;
    }
    output
}


fn main() {
    // 1. Build the schema: multiple fields
    // Example: FloatCol (float32), IntCol (int32), StrCol (string)
    // Bond type ids: BT_FLOAT = 7, BT_INT32 = 16, BT_STRING = 9
    let fields = &[
        ("FloatCol", 8u8, 1u16),   // float
        ("IntCol",   16u8, 2u16),  // int32
        ("StrCol",   9u8, 3u16),   // string
    ];
    let schema_obj = BondSchema::from_fields(fields);
    let schema_bytes = schema_obj.as_bytes().to_vec();
    let schema_md5 = md5_bytes(&schema_bytes);
    let schema_id = 1u64; // arbitrary

    let schema = CentralSchemaEntry {
        id: schema_id,
        md5: schema_md5,
        schema: schema_obj,
    };

    // 2. Build the row with multiple values (must match schema order/types exactly)
    let mut row = Vec::new();
    row.extend_from_slice(&3.1415f64.to_le_bytes()); // FloatCol (float32)
    row.extend_from_slice(&42i32.to_le_bytes());     // IntCol (int32)
    let s = "hello";
    row.extend_from_slice(&(s.len() as u16).to_le_bytes()); // StrCol: string length (u16 LE)
    row.extend_from_slice(s.as_bytes());                    // StrCol: string bytes

    let row_obj = BondRow::from_schema_and_row(&schema.schema, &row);

    let event = CentralEventEntry {
        schema_id,
        level: 1,
        event_name: "basename".to_string(),
        row: row_obj,
    };

    // 3. Metadata
    let metadata = "namespace=testNamespace/eventVersion=Ver1v0/tenant=T/role=R/roleinstance=RI";

    // 4. Build central bond blob
    let blob = CentralBondBlob {
        version: 1,
        format: 2,
        metadata: metadata.to_string(),
        schemas: vec![schema],
        events: vec![event],
    };

    let payload = blob.to_bytes();
    File::create("/tmp/rust_central_bond_blob_multi.uncompressed")
        .unwrap()
        .write_all(&payload)
        .unwrap();

    // LZ4 compress and save
    let compressed = lz4_chunked_compress(&payload);
    File::create("/tmp/rust_central_bond_blob_multi.lz4")
        .unwrap()
        .write_all(&compressed)
        .unwrap();

    println!(
        "Wrote /tmp/rust_central_bond_blob_multi.uncompressed ({} bytes) and .lz4 ({} bytes)",
        payload.len(),
        compressed.len()
    );
}