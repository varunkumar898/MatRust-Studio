pub fn fuzz_one(data: &[u8]) {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = mlapp_ui::parse_app_xml(s);
        let _ = mlapp_lang::extract_code_from_document_xml(s);
    }
}

#[cfg(not(target_os = "windows"))]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});

#[cfg(target_os = "windows")]
fn main() {
    println!("Fuzzing on Windows runs via smoke test runner.");
}
