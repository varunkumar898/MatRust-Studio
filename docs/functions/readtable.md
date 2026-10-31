# `readtable`

Read tabular data from file into table struct.

## Syntax

```matlab
T = readtable(filename)
```

## Description

`T = readtable(filename)` creates a table by reading column-oriented data from a text file (such as `.csv`, `.tsv`, or `.txt`).

The table is returned as a structured representation with fields corresponding to each column name. Column variables can be directly accessed via standard dot notation:
```matlab
T = readtable("data.csv");
temperature = T.temperature;
```

## Implementation Details

- Automatically detects comma, semicolon, or tab delimiters.
- Strips UTF-8 byte order mark (BOM) if present.
- Detects column types: columns with numbers evaluate to `Double` column vectors; text columns evaluate to `String` arrays.
- Empties and NaN entries are mapped to IEEE-754 `NaN`.
