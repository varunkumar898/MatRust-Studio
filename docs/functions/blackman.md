# `blackman`

Blackman window.

## Syntax

```matlab
w = blackman(N)
```

## Description

`w = blackman(N)` returns an $N$-point symmetric Blackman window vector:
$$w(n) = 0.42 - 0.5 \cos\left(\frac{2\pi n}{N - 1}\right) + 0.08 \cos\left(\frac{4\pi n}{N - 1}\right)$$
