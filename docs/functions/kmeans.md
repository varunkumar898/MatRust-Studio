# `kmeans`

k-means clustering.

## Syntax

```matlab
[idx, C] = kmeans(X, k)
```

## Description

`[idx, C] = kmeans(X, k)` partitions the observations in the $N \times p$ data matrix `X` into `k` clusters. Returns cluster indices `idx` ($N \times 1$) and centroid coordinates `C` ($k \times p$).
