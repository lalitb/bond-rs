use bond_rs::encoder::{BondEncoder, BondField};
use std::fs::File;
use std::io::Write;

fn main() {
    // Create a new Bond encoder
    let mut encoder = BondEncoder::new();

    // OPTION 1: Using field array (most efficient)
    let fields = [
        BondField::float("FloatCol", 3.1415),
        BondField::int32("IntCol", 42),
        BondField::string("StrCol", "hello"),
    ];

    // Metadata
    let metadata = "namespace=testNamespace/eventVersion=Ver1v0/tenant=T/role=R/roleinstance=RI";

    // Encode the data into a Bond blob
    let payload = encoder.encode(&fields, "basename", 1, metadata);

    // OPTION 2: Using builder pattern (more ergonomic)
    let _payload2 = encoder
        .builder()
        .add_float("FloatCol", 3.1415)
        .add_int32("IntCol", 42)
        .add_string("StrCol", "hello")
        .build("basename", 1, metadata);

    // Write to file
    File::create("/tmp/rust_central_bond_blob_simple.uncompressed")
        .unwrap()
        .write_all(&payload)
        .unwrap();

    // LZ4 compress and save
    let compressed = lz4_chunked_compress(&payload);
    File::create("/tmp/rust_central_bond_blob_simple.lz4")
        .unwrap()
        .write_all(&compressed)
        .unwrap();

    println!(
        "Wrote /tmp/rust_central_bond_blob_simple.uncompressed ({} bytes) and .lz4 ({} bytes)",
        payload.len(),
        compressed.len()
    );
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
