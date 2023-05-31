use shrine::file_format::ShrineFile;
use shrine::serialize::bson::BsonSerDe;
use shrine::serialize::json::JsonSerDe;
use shrine::shrine::Shrine;
use std::io::{stdout, Write};

fn main() {
    let mut shrine = Shrine::new();
    shrine.set("key", "val");

    let json_serde = Box::new(JsonSerDe::new());
    let _ = stdout().write_all(shrine.as_bytes(json_serde).unwrap().as_slice());

    let bson_serde = Box::new(BsonSerDe::new());
    let _ = stdout().write_all(shrine.as_bytes(bson_serde).unwrap().as_slice());

    let mp_serde = Box::new(BsonSerDe::new());
    let _ = stdout().write_all(shrine.as_bytes(mp_serde).unwrap().as_slice());

    let bytes = ShrineFile::default().as_bytes().unwrap();
    let bytes = bytes.as_slice();
    let _ = stdout().write_all(bytes);
}
