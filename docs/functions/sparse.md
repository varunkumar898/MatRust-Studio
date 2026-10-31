# `sparse`

Construct Compressed Sparse Column (CSC) matrix.

## Syntax
```matlab
S = sparse(A)
S = sparse(m, n)
S = sparse(i, j, v)
S = sparse(i, j, v, m, n)
```

## Description
Constructs a 2D sparse matrix stored in Compressed Sparse Column (CSC) format.
When provided coordinate triplets `(i, j, v)`, duplicate coordinates are automatically accumulated by summation.

## Examples
```matlab
i = [1 2 2 3];
j = [1 2 2 3];
v = [10 20 30 40];
S = sparse(i, j, v, 3, 3);
% S(2, 2) is 50 due to duplicate accumulation (20 + 30)
```
