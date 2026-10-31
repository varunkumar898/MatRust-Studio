//! Classical machine learning builtins: linear regression (fitlm, predict), k-means clustering, confusion matrix.

use crate::helpers::user_err;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, StructArray, Value};
use std::collections::HashMap;

fn extract_f64_matrix(val: &Value) -> Result<(Vec<usize>, Vec<f64>), EvalError> {
    match val {
        Value::Double(d) => Ok((d.shape().to_vec(), d.as_slice().to_vec())),
        Value::Single(s) => Ok((
            s.shape().to_vec(),
            s.as_slice().iter().map(|&x| x as f64).collect(),
        )),
        _ => Err(user_err("RUSTLAB:invalidType", "Expected numeric matrix.")),
    }
}

/// Fits a linear regression model: model = fitlm(X, y).
pub fn fitlm_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "fitlm requires at least 2 inputs (X, y).",
        ));
    }
    let (x_shp, x_data) = extract_f64_matrix(&args[0])?;
    let (_y_shp, y_data) = extract_f64_matrix(&args[1])?;

    let n = x_shp.first().copied().unwrap_or(0);
    let p = x_shp.get(1).copied().unwrap_or(1);

    if n != y_data.len() || n == 0 {
        return Err(user_err(
            "RUSTLAB:fitlm:lengthMismatch",
            "Number of rows in X must match length of y.",
        ));
    }

    let p_aug = p + 1; // Intercept + p features
                       // Build X_aug (N x (p+1)) with column 0 = 1.0 (intercept)
                       // Column-major: element at row r, col c is at r + c * n
    let mut x_aug = vec![0.0; n * p_aug];
    for r in 0..n {
        x_aug[r] = 1.0; // Column 0: intercept
    }
    for c in 0..p {
        for r in 0..n {
            x_aug[r + (c + 1) * n] = x_data[r + c * n];
        }
    }

    // Normal equations: (X^T * X) * beta = X^T * y
    // Compute A = X^T * X (p_aug x p_aug)
    let mut a_mat = vec![0.0; p_aug * p_aug];
    for r in 0..p_aug {
        for c in 0..p_aug {
            let mut sum = 0.0;
            for k in 0..n {
                sum += x_aug[k + r * n] * x_aug[k + c * n];
            }
            a_mat[r + c * p_aug] = sum;
        }
    }

    // Compute b = X^T * y (p_aug x 1)
    let mut b_vec = vec![0.0; p_aug];
    for r in 0..p_aug {
        let mut sum = 0.0;
        for k in 0..n {
            sum += x_aug[k + r * n] * y_data[k];
        }
        b_vec[r] = sum;
    }

    // Solve A * beta = b via Gaussian elimination with partial pivoting
    let mut a_work = a_mat;
    let mut b_work = b_vec;
    for k in 0..p_aug {
        let mut max_row = k;
        let mut max_val = a_work[k + k * p_aug].abs();
        for r in (k + 1)..p_aug {
            let val = a_work[r + k * p_aug].abs();
            if val > max_val {
                max_val = val;
                max_row = r;
            }
        }
        if max_val < 1e-15 {
            return Err(user_err(
                "RUSTLAB:fitlm:singularMatrix",
                "Design matrix X is rank-deficient or singular.",
            ));
        }
        if max_row != k {
            for c in 0..p_aug {
                a_work.swap(k + c * p_aug, max_row + c * p_aug);
            }
            b_work.swap(k, max_row);
        }

        let pivot = a_work[k + k * p_aug];
        for r in (k + 1)..p_aug {
            let factor = a_work[r + k * p_aug] / pivot;
            for c in (k + 1)..p_aug {
                a_work[r + c * p_aug] -= factor * a_work[k + c * p_aug];
            }
            b_work[r] -= factor * b_work[k];
        }
    }

    // Back substitution
    let mut beta = vec![0.0; p_aug];
    for k in (0..p_aug).rev() {
        let mut sum = b_work[k];
        for c in (k + 1)..p_aug {
            sum -= a_work[k + c * p_aug] * beta[c];
        }
        beta[k] = sum / a_work[k + k * p_aug];
    }

    // Compute predictions, residuals, R^2, RMSE
    let mut y_pred = Vec::with_capacity(n);
    let mut residuals = Vec::with_capacity(n);
    let mut ss_res = 0.0;
    let y_mean = y_data.iter().sum::<f64>() / (n as f64);
    let mut ss_tot = 0.0;

    for r in 0..n {
        let mut pred = 0.0;
        for c in 0..p_aug {
            pred += x_aug[r + c * n] * beta[c];
        }
        y_pred.push(pred);
        let res = y_data[r] - pred;
        residuals.push(res);
        ss_res += res * res;
        let diff = y_data[r] - y_mean;
        ss_tot += diff * diff;
    }

    let r_squared = if ss_tot > 0.0 {
        1.0 - (ss_res / ss_tot)
    } else {
        1.0
    };
    let rmse = (ss_res / (n as f64)).sqrt();

    let mut model_map = HashMap::new();
    let beta_da = DenseArray::from_vec_unchecked(vec![p_aug, 1], beta);
    let res_da = DenseArray::from_vec_unchecked(vec![n, 1], residuals);

    model_map.insert("Coefficients".to_string(), Value::Double(beta_da));
    model_map.insert("Rsquared".to_string(), Value::scalar(r_squared));
    model_map.insert("RMSE".to_string(), Value::scalar(rmse));
    model_map.insert("Residuals".to_string(), Value::Double(res_da));
    model_map.insert("type".to_string(), Value::string_scalar("LinearModel"));

    let fields = vec![
        "Coefficients".to_string(),
        "Rsquared".to_string(),
        "RMSE".to_string(),
        "Residuals".to_string(),
        "type".to_string(),
    ];
    let sa = StructArray::new(vec![1, 1], fields, vec![model_map]);
    Ok(vec![Value::Struct(sa)])
}

/// Evaluates linear model predictions: y_pred = predict(model, X).
pub fn predict_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "predict requires at least 2 inputs (model, X).",
        ));
    }
    let beta = match &args[0] {
        Value::Struct(sa) => {
            if sa.data.is_empty() {
                return Err(user_err(
                    "RUSTLAB:predict:invalidModel",
                    "Invalid model struct.",
                ));
            }
            let map = &sa.data[0];
            let coef_val = map.get("Coefficients").ok_or_else(|| {
                user_err(
                    "RUSTLAB:predict:invalidModel",
                    "Missing 'Coefficients' field.",
                )
            })?;
            match coef_val {
                Value::Double(d) => d.as_slice().to_vec(),
                _ => {
                    return Err(user_err(
                        "RUSTLAB:predict:invalidModel",
                        "Coefficients must be double.",
                    ))
                }
            }
        }
        _ => {
            return Err(user_err(
                "RUSTLAB:predict:invalidModel",
                "First argument must be a trained model struct from fitlm.",
            ))
        }
    };

    let (x_shp, x_data) = extract_f64_matrix(&args[1])?;
    let n = x_shp.first().copied().unwrap_or(0);
    let p = x_shp.get(1).copied().unwrap_or(1);

    if beta.len() != p + 1 {
        return Err(user_err(
            "RUSTLAB:predict:featureMismatch",
            &format!(
                "Model expects {} features, but X has {}.",
                beta.len() - 1,
                p
            ),
        ));
    }

    let mut y_pred = Vec::with_capacity(n);
    for r in 0..n {
        let mut pred = beta[0]; // Intercept
        for c in 0..p {
            pred += x_data[r + c * n] * beta[c + 1];
        }
        y_pred.push(pred);
    }

    let res_da = DenseArray::from_vec_unchecked(vec![n, 1], y_pred);
    Ok(vec![Value::Double(res_da)])
}

/// K-means clustering: [idx, C] = kmeans(X, k).
pub fn kmeans_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "kmeans requires at least 2 inputs (X, k).",
        ));
    }
    let (x_shp, x_data) = extract_f64_matrix(&args[0])?;
    let k = args[1].as_scalar_f64().map_err(|_| {
        user_err(
            "RUSTLAB:kmeans:invalidK",
            "k must be a positive integer scalar.",
        )
    })? as usize;

    let n = x_shp.first().copied().unwrap_or(0);
    let p = x_shp.get(1).copied().unwrap_or(1);

    if n < k || k == 0 {
        return Err(user_err(
            "RUSTLAB:kmeans:tooFewDataPoints",
            "Number of points n must be at least k.",
        ));
    }

    // Deterministic centroid initialization: evenly spaced points from dataset
    let mut centroids = vec![0.0; k * p];
    let step = n / k;
    for c_idx in 0..k {
        let sample_r = (c_idx * step).min(n - 1);
        for col in 0..p {
            centroids[c_idx + col * k] = x_data[sample_r + col * n];
        }
    }

    let mut assignments = vec![1.0; n];
    let max_iter = 100;

    for _ in 0..max_iter {
        let mut changed = false;

        // Step 1: Assign each point to nearest centroid
        for r in 0..n {
            let mut min_dist = f64::INFINITY;
            let mut best_c = 0;
            for c_idx in 0..k {
                let mut dist_sq = 0.0;
                for col in 0..p {
                    let diff = x_data[r + col * n] - centroids[c_idx + col * k];
                    dist_sq += diff * diff;
                }
                if dist_sq < min_dist {
                    min_dist = dist_sq;
                    best_c = c_idx;
                }
            }
            let new_label = (best_c + 1) as f64;
            if (assignments[r] - new_label).abs() > 0.5 {
                assignments[r] = new_label;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        // Step 2: Recompute centroid positions
        let mut counts = vec![0usize; k];
        let mut new_centroids = vec![0.0; k * p];
        for r in 0..n {
            let c_idx = (assignments[r] as usize) - 1;
            counts[c_idx] += 1;
            for col in 0..p {
                new_centroids[c_idx + col * k] += x_data[r + col * n];
            }
        }
        for c_idx in 0..k {
            if counts[c_idx] > 0 {
                for col in 0..p {
                    new_centroids[c_idx + col * k] /= counts[c_idx] as f64;
                }
            }
        }
        centroids = new_centroids;
    }

    let idx_da = DenseArray::from_vec_unchecked(vec![n, 1], assignments);
    let c_da = DenseArray::from_vec_unchecked(vec![k, p], centroids);

    if nargout <= 1 {
        Ok(vec![Value::Double(idx_da)])
    } else {
        Ok(vec![Value::Double(idx_da), Value::Double(c_da)])
    }
}

/// Calculates confusion matrix: C = confusionmat(group, grouphat).
pub fn confusionmat_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "confusionmat requires at least 2 inputs (group, grouphat).",
        ));
    }
    let (_, g1) = extract_f64_matrix(&args[0])?;
    let (_, g2) = extract_f64_matrix(&args[1])?;

    if g1.len() != g2.len() {
        return Err(user_err(
            "RUSTLAB:confusionmat:lengthMismatch",
            "Group vectors must have the same length.",
        ));
    }

    let mut classes: Vec<i64> = g1
        .iter()
        .chain(g2.iter())
        .map(|&x| x.round() as i64)
        .collect();
    classes.sort();
    classes.dedup();

    let num_classes = classes.len();
    let mut mat = vec![0.0; num_classes * num_classes];

    for i in 0..g1.len() {
        let true_c = g1[i].round() as i64;
        let pred_c = g2[i].round() as i64;
        if let (Some(r), Some(c)) = (
            classes.iter().position(|&x| x == true_c),
            classes.iter().position(|&x| x == pred_c),
        ) {
            mat[r + c * num_classes] += 1.0;
        }
    }

    let da = DenseArray::from_vec_unchecked(vec![num_classes, num_classes], mat);
    Ok(vec![Value::Double(da)])
}
