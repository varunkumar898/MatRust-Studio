use std::io::Cursor;

pub fn fuzz_one(data: &[u8]) {
    let cursor = Cursor::new(data);
    let _ = mlapp_container::AppContainer::open(cursor);
}

#[cfg(not(target_os = "windows"))]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});

#[cfg(target_os = "windows")]
fn main() {
    println!("Fuzzing on Windows runs via smoke test runner.");
}
