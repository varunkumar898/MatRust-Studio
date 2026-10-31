# `kron`

Kronecker tensor product.

## Syntax

```matlab
K = kron(A, B)
```

## Description

- `K = kron(A, B)` returns the Kronecker tensor product of matrices `A` and `B`.
- If `A` is $m_A \times n_A$ and `B` is $m_B \times n_B$, then `K` is $(m_A \cdot m_B) \times (n_A \cdot n_B)$ where block $(i, j)$ is $A_{ij} \cdot B$.
- Operates on real and complex dense arrays.
