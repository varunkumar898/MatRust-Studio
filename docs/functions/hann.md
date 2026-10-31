# `hann`

Hann window.

## Syntax

```matlab
w = hann(N)
```

## Description

`w = hann(N)` returns an $N$-point symmetric Hann window vector:
$$w(n) = 0.5 \left(1 - \cos\left(\frac{2\pi n}{N - 1}\right)\right)$$
