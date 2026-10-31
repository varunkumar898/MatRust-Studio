# `fitlm`

Fit linear regression model.

## Syntax

```matlab
model = fitlm(X, y)
```

## Description

`model = fitlm(X, y)` returns a full ordinary least-squares linear regression model fit to predictor matrix `X` ($N \times p$) and response vector `y` ($N \times 1$).

The returned structure contains model properties:
- `Coefficients`: Estimated parameter vector $[\beta_0, \beta_1, \dots, \beta_p]^T$.
- `Rsquared`: Coefficient of determination $R^2$.
- `RMSE`: Root mean square error of the residuals.
- `Residuals`: Vector of differences $y - \hat{y}$.
