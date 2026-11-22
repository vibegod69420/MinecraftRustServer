use std::io::Cursor;
use std::path::Path;

use mc_nbt::{NbtTag, read_nbt, read_nbt_compressed, write_nbt, write_nbt_compressed};
use serde_json::Value;

#[test]
fn test_nbt_golden_vectors() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/vectors/nbt.json");
    if !p.exists() {
        eprintln!("[skip] nbt.json not present");
        return;
    }
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let raw = hex::decode(v["uncompressed_hex"].as_str().unwrap()).unwrap();

    let (root_name, root_tag) = read_nbt(&mut Cursor::new(&raw)).unwrap();
    assert_eq!(root_name, "");
    assert_eq!(root_tag.compound_get("b"), Some(&NbtTag::Byte(-42)));
    assert_eq!(root_tag.compound_get("s"), Some(&NbtTag::Short(-12345)));
    assert_eq!(root_tag.compound_get("i"), Some(&NbtTag::Int(0x76543210)));
    assert_eq!(root_tag.compound_get("l"), Some(&NbtTag::Long(-1234567890123456789)));
    // Java FloatTag.valueOf(-0.0F) checks `value == 0.0F ? ZERO : new FloatTag(value)`
    // so -0.0F is canonicalized to FloatTag.ZERO (+0.0F).
    match root_tag.compound_get("f_neg_zero") {
        Some(NbtTag::Float(f)) => assert_eq!(f.to_bits(), 0.0_f32.to_bits()),
        other => panic!("expected Float(+0.0 canonicalized from -0.0), got {other:?}"),
    }
    assert_eq!(
        root_tag.compound_get("str_utf8"),
        Some(&NbtTag::String(v["str_with_nul_and_emoji"].as_str().unwrap().to_string()))
    );

    // Round-trip uncompressed & verify byte-exact equality with Java NbtIo output!
    let mut re_encoded = Vec::new();
    write_nbt(&mut re_encoded, &root_name, &root_tag).unwrap();
    assert_eq!(re_encoded, raw);

    // Round-trip compressed & verify structural equality!
    let gz = write_nbt_compressed("", &root_tag).unwrap();
    let (_, back) = read_nbt_compressed(&gz).unwrap();
    assert!(root_tag.structural_eq(&back));
    assert_eq!(root_tag.canonical_sha256(), back.canonical_sha256());
}
