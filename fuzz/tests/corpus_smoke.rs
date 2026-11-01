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
