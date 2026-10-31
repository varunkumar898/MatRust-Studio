# `writematrix`

Write matrix to a delimited file.

## Syntax

```matlab
writematrix(A, filename)
writematrix(A, filename, 'Delimiter', d)
```

## Description

- `writematrix(A, filename)` writes a 2-D numeric matrix to a delimited text file.
- Formats floating-point numbers with shortest round-trip accuracy via `ryu`.
- Clean-room implementation in `rustlab-data` using the `csv` crate.

## References

1. MATLAB Documentation: `writematrix`
2. RFC 4180: Common Format and MIME Type for CSV Files
