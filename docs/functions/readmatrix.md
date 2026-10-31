# `readmatrix`

Read matrix from a delimited file.

## Syntax

```matlab
M = readmatrix(filename)
M = readmatrix(filename, 'Delimiter', d)
M = readmatrix(filename, 'NumHeaderLines', n)
```

## Description

- `M = readmatrix(filename)` reads a 2-D numeric matrix from a CSV or delimited text file.
- Automatically handles UTF-8 BOM, CRLF/LF line endings, and empty cells (converted to NaN).
- Detects ragged rows and reports informative error with line number.
- Clean-room implementation in `rustlab-data` using the `csv` crate.

## References

1. MATLAB Documentation: `readmatrix`
2. RFC 4180: Common Format and MIME Type for CSV Files
