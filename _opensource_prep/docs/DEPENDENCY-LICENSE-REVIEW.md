# Dependency license review (proposal)

The current `cargo deny check licenses` run exited successfully with zero
errors, two warnings, and 286 notes. The warnings are unused `CC0-1.0` and
`Unicode-DFS-2016` allowances in `deny.toml`. The inventory found no GPL-family
license.

These packages use licenses outside the explicitly named MIT/Apache/BSD/ISC/
Unicode/Zlib set:

| License | Packages in the locked inventory | Plain-language note |
|---|---|---|
| BSL-1.0 | `clipboard-win` 5.4.3, `error-code` 3.4.0, `ryu` 1.0.23 | A permissive license, but its terms are not one of the project’s listed license choices. Review required notices and distribution terms. |
| OFL-1.1 | `epaint_default_fonts` 0.31.1 | Font license; font files have their own redistribution and attribution terms. |
| Ubuntu-font-1.0 | `epaint_default_fonts` 0.31.1 | Separate Ubuntu font license; inspect the actual bundled font assets and notices before redistributing binaries. |
| Unlicense | `aho-corasick` 1.1.5, `byteorder-lite` 0.1.0, `csv` 1.4.0, `csv-core` 0.1.13, `memchr` 2.8.3 | A permissive public-domain dedication/license, but not one of the explicitly named identifiers. |

`0BSD` also appears for `adler2` 2.0.1, alongside Apache-2.0 and MIT. It is
BSD-style and is covered if “BSD” in the project policy includes 0BSD.

None of these findings is GPL, so the inventory does not contradict a narrow
“no GPL dependencies identified” statement. They do mean that “all
dependencies are MIT/Apache/BSD/ISC/Unicode/Zlib” would be inaccurate.
Third-party dependency terms do not automatically change the license offered
for this project’s own code under MIT OR Apache-2.0, but binary redistributors
must preserve applicable dependency/font notices and satisfy each license.
Have the maintainer review the complete dependency and asset notices before
publishing binaries; this is not legal advice.

Suggested public wording:

> MatRust Studio source code is offered under either MIT or Apache-2.0. Third-
> party dependencies and bundled assets retain their own licenses; see the
> dependency and asset notices for details. No GPL-family license was found in
> the current locked dependency inventory.

The last sentence is time- and inventory-specific. Re-run the license audit
after dependency changes and before each release.
