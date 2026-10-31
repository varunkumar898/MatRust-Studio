# `summary`

Summary statistics of table variables.

## Syntax

```matlab
s = summary(T)
```

## Description

`s = summary(T)` computes summary statistics (Min, Mean, Max) for each numeric variable in table `T`.

## Implementation Details

- Iterates through column variables in `T`.
- Computes minimum, arithmetic mean, and maximum omitting NaN values.
- Returns a structured summary array.
