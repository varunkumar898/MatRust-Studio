# `hamming`

Hamming window.

## Syntax

```matlab
w = hamming(N)
```

## Description

`w = hamming(N)` returns an $N$-point symmetric Hamming window vector:
$$w(n) = 0.54 - 0.46 \cos\left(\frac{2\pi n}{N - 1}\right)$$
