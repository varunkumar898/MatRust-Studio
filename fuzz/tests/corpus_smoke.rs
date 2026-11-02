//! Smoke and regression tests validating the seed corpus against target fuzz harnesses.

use std::fs;
use std::io::Cursor;
use std::path::Path;

#[test]
fn test_seed_corpus_smoke_parser() {
    let corpus_dir = Path::new("corpus/fuzz_parser");
    if let Ok(entries) = fs::read_dir(corpus_dir) {
        for entry in entries.flatten() {
            if let Ok(data) = fs::read(entry.path()) {
                if let Ok(s) = std::str::from_utf8(&data) {
                    let _ = rustlab_parse::parse(s);
                    let _ = mlapp_lang::parse_app_classdef(s);
                }
            }
        }
    }

    // Direct smoke test cases for edge-case syntax
    let edge_cases: &[&[u8]] = &[
        b"",
        b"1+1",
        b"A = [1 2; 3 4]'",
        b"for i = 1:10; x(i) = i^2; end",
        b"classdef App < matlab.apps.AppBase\nproperties\nx\nend\nmethods\nfunction startup(app)\nend\nend\nend",
        &[0xff, 0xfe, 0x00, 0x01],
        &[b'('; 500],
        &[b'['; 500],
        &[b'{'; 500],
    ];

    for &input in edge_cases {
        if let Ok(s) = std::str::from_utf8(input) {
            let _ = rustlab_parse::parse(s);
            let _ = mlapp_lang::parse_app_classdef(s);
        }
    }
}

#[test]
fn test_seed_corpus_smoke_xml() {
    let corpus_dir = Path::new("corpus/fuzz_xml");
    if let Ok(entries) = fs::read_dir(corpus_dir) {
        for entry in entries.flatten() {
            if let Ok(data) = fs::read(entry.path()) {
                if let Ok(s) = std::str::from_utf8(&data) {
                    let _ = mlapp_ui::parse_app_xml(s);
                    let _ = mlapp_lang::extract_code_from_document_xml(s);
                }
            }
        }
    }

    let edge_cases: &[&str] = &[
        "",
        "<root></root>",
        "<?xml version=\"1.0\"?><App><UIFigure/></App>",
        "<malformed <><> &",
        "<a b=\"c\"><![CDATA[code]]></a>",
    ];

    for &s in edge_cases {
        let _ = mlapp_ui::parse_app_xml(s);
        let _ = mlapp_lang::extract_code_from_document_xml(s);
    }
}

#[test]
fn test_seed_corpus_smoke_zip() {
    let corpus_dir = Path::new("corpus/fuzz_zip");
    if let Ok(entries) = fs::read_dir(corpus_dir) {
        for entry in entries.flatten() {
            if let Ok(data) = fs::read(entry.path()) {
                let cursor = Cursor::new(data);
                let _ = mlapp_container::AppContainer::open(cursor);
            }
        }
    }

    let edge_cases: &[&[u8]] = &[
        b"",
        b"PK\x03\x04",
        b"PK\x05\x06\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        &[0u8; 1024],
        &[0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00],
    ];

    for &bytes in edge_cases {
        let cursor = Cursor::new(bytes);
        let _ = mlapp_container::AppContainer::open(cursor);
    }
}

#[test]
fn test_seed_corpus_smoke_container_extract() {
    let corpus_dir = Path::new("corpus/fuzz_container_extract");
    if let Ok(entries) = fs::read_dir(corpus_dir) {
        for entry in entries.flatten() {
            if let Ok(data) = fs::read(entry.path()) {
                let cursor = Cursor::new(data);
                if let Ok(mut container) = mlapp_container::AppContainer::open(cursor) {
                    if let Ok(tmp) = tempfile::tempdir() {
                        let _ = container.extract_all(tmp.path());
                    }
                }
            }
        }
    }

    // Edge cases for container extraction
    let edge_cases: &[&[u8]] = &[
        b"",
        b"PK\x03\x04",
        b"PK\x05\x06\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        &[0u8; 2048],
    ];

    for &bytes in edge_cases {
        let cursor = Cursor::new(bytes);
        if let Ok(mut container) = mlapp_container::AppContainer::open(cursor) {
            if let Ok(tmp) = tempfile::tempdir() {
                let _ = container.extract_all(tmp.path());
            }
        }
    }
}

#[test]
fn test_seed_corpus_smoke_interpreter() {
    let corpus_dir = Path::new("corpus/fuzz_interpreter");
    if let Ok(entries) = fs::read_dir(corpus_dir) {
        for entry in entries.flatten() {
            if let Ok(data) = fs::read(entry.path()) {
                if let Ok(s) = std::str::from_utf8(&data) {
                    let mut interp = rustlab_eval::Interpreter::new();
                    rustlab_stdlib::register_all(&mut interp);
                    interp.max_recursion_depth = 16;
                    let _ = interp.eval_script(s);
                }
            }
        }
    }

    // Edge cases including loop constructs, expressions, and syntax edge cases
    let edge_cases: &[&str] = &[
        "",
        "x = 1 + 2;",
        "s = 0; for i = 1:5; s = s + i; end",
        "while false; x = 1; end",
        "A = [1 2; 3 4]; B = A * [5; 6];",
        "try; x = 1/0; catch ME; y = 2; end",
        "f = @(x) x + 1; z = f(10);",
    ];

    for &s in edge_cases {
        let mut interp = rustlab_eval::Interpreter::new();
        rustlab_stdlib::register_all(&mut interp);
        interp.max_recursion_depth = 16;
        let _ = interp.eval_script(s);
    }
}
