//! G0.1-VERIFY regression test for RustLab-only error identifiers.
//!
//! Milestone G0 renamed RustLab-only identifiers to the `RUSTLAB:` namespace.
//! This test scans the two data modules that own those identifiers and rejects
//! a `MATLAB:` prefix unless the identifier is a generic MATLAB identifier we
//! intentionally retain. Add new generic MATLAB identifiers to the allow-list
//! only together with a catalog entry in `docs/spec/error-catalog.md`.

const DATA_STATS: &str = include_str!("../src/stats.rs");
const DATA_CSV_IO: &str = include_str!("../src/csv_io.rs");

/// Generic MATLAB identifiers retained in the data crate. They describe
/// argument-count errors, not RustLab-only behavior, and are documented in
/// `docs/spec/error-catalog.md` as UNVERIFIED identifier strings.
const ALLOWED_MATLAB_IDS: &[&str] = &["MATLAB:minrhs", "MATLAB:maxrhs"];

fn quoted_identifiers(source: &str) -> Vec<&str> {
    source
        .split('"')
        .enumerate()
        .filter_map(|(index, part)| {
            if index % 2 == 1 && (part.starts_with("MATLAB:") || part.starts_with("RUSTLAB:")) {
                Some(part)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn test_g01_rustlab_only_ids_do_not_use_matlab_prefix() {
    let mut saw_rustlab_id = false;

    for (file, source) in [("stats.rs", DATA_STATS), ("csv_io.rs", DATA_CSV_IO)] {
        for id in quoted_identifiers(source) {
            if id.starts_with("MATLAB:") {
                assert!(
                    ALLOWED_MATLAB_IDS.contains(&id),
                    "{file}: RustLab-only identifier {id} uses the MATLAB: prefix; use RUSTLAB:"
                );
            } else if id.starts_with("RUSTLAB:") {
                saw_rustlab_id = true;
            }
        }
    }

    assert!(
        saw_rustlab_id,
        "the scan found no RUSTLAB: identifiers; check the source paths"
    );
}
