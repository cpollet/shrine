use shrine::file_format::ShrineFile;
use std::io::{stdout, Write};

fn main() {
    let bytes = ShrineFile::default().as_bytes().unwrap();
    let bytes = bytes.as_slice();
    let _ = stdout().write_all(bytes);
}
