# `writetable`

Write table struct to delimited file.

## Syntax

```matlab
writetable(T, filename)
```

## Description

`writetable(T, filename)` writes table or structure `T` to a comma-delimited text file specified by `filename`.

## Implementation Details

- Writes header line containing sanitized column names.
- Iterates row by row across numeric and string column vectors.
- Flushes buffer to disk upon completion.
