//! Image processing builtins: rgb2gray, imresize, imfilter.

use crate::helpers::user_err;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

/// Converts RGB image to grayscale: I = rgb2gray(RGB).
pub fn rgb2gray_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "rgb2gray requires 1 input."));
    }
    match &args[0] {
        Value::Double(d) => {
            let shp = d.shape();
            if shp.len() < 3 || shp[2] != 3 {
                return Err(user_err(
                    "MATLAB:rgb2gray:invalidSizeRGB",
                    "Input RGB image must be HxWx3 array.",
                ));
            }
            let h = shp[0];
            let w = shp[1];
            let plane_size = h * w;
            let slice = d.as_slice();

            let mut gray = Vec::with_capacity(plane_size);
            for i in 0..plane_size {
                let r = slice[i];
                let g = slice[i + plane_size];
                let b = slice[i + 2 * plane_size];
                let lum = 0.2989 * r + 0.5870 * g + 0.1140 * b;
                gray.push(lum);
            }

            let da = DenseArray::from_vec_unchecked(vec![h, w], gray);
            Ok(vec![Value::Double(da)])
        }
        Value::UInt8(u) => {
            let shp = u.shape();
            if shp.len() < 3 || shp[2] != 3 {
                return Err(user_err(
                    "MATLAB:rgb2gray:invalidSizeRGB",
                    "Input RGB image must be HxWx3 array.",
                ));
            }
            let h = shp[0];
            let w = shp[1];
            let plane_size = h * w;
            let slice = u.as_slice();

            let mut gray = Vec::with_capacity(plane_size);
            for i in 0..plane_size {
                let r = slice[i] as f64;
                let g = slice[i + plane_size] as f64;
                let b = slice[i + 2 * plane_size] as f64;
                let lum = (0.2989 * r + 0.5870 * g + 0.1140 * b)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                gray.push(lum);
            }

            let da = DenseArray::from_vec_unchecked(vec![h, w], gray);
            Ok(vec![Value::UInt8(da)])
        }
        _ => Err(user_err(
            "MATLAB:rgb2gray:invalidType",
            "Input must be uint8 or double array.",
        )),
    }
}

/// Resizes 2D image array: J = imresize(I, scale).
pub fn imresize_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "imresize requires at least 2 inputs (I, scale).",
        ));
    }
    let scale = args[1]
        .as_scalar_f64()
        .map_err(|_| user_err("MATLAB:imresize:invalidScale", "Scale must be a scalar."))?;
    if scale <= 0.0 {
        return Err(user_err(
            "MATLAB:imresize:invalidScale",
            "Scale must be positive.",
        ));
    }

    match &args[0] {
        Value::Double(d) => {
            let shp = d.shape();
            let h = shp.first().copied().unwrap_or(0);
            let w = shp.get(1).copied().unwrap_or(0);
            if h == 0 || w == 0 {
                return Ok(vec![Value::Double(DenseArray::empty())]);
            }

            let new_h = ((h as f64) * scale).round().max(1.0) as usize;
            let new_w = ((w as f64) * scale).round().max(1.0) as usize;
            let mut out = Vec::with_capacity(new_h * new_w);

            for c in 0..new_w {
                let orig_c = ((c as f64) / scale).floor().min((w - 1) as f64) as usize;
                for r in 0..new_h {
                    let orig_r = ((r as f64) / scale).floor().min((h - 1) as f64) as usize;
                    out.push(d.as_slice()[orig_r + orig_c * h]);
                }
            }

            let da = DenseArray::from_vec_unchecked(vec![new_h, new_w], out);
            Ok(vec![Value::Double(da)])
        }
        _ => Err(user_err(
            "MATLAB:imresize:invalidType",
            "Currently only 2D double matrices are supported.",
        )),
    }
}

/// 2D spatial image filtering: J = imfilter(I, h).
pub fn imfilter_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "imfilter requires at least 2 inputs (I, h).",
        ));
    }
    match (&args[0], &args[1]) {
        (Value::Double(im), Value::Double(kernel)) => {
            let im_shp = im.shape();
            let k_shp = kernel.shape();
            let h = im_shp.first().copied().unwrap_or(0);
            let w = im_shp.get(1).copied().unwrap_or(0);
            let kh = k_shp.first().copied().unwrap_or(0);
            let kw = k_shp.get(1).copied().unwrap_or(0);

            if h == 0 || w == 0 || kh == 0 || kw == 0 {
                return Ok(vec![Value::Double(DenseArray::empty())]);
            }

            let pad_h = (kh / 2) as isize;
            let pad_w = (kw / 2) as isize;
            let mut out = Vec::with_capacity(h * w);

            for c in 0..w {
                for r in 0..h {
                    let mut sum = 0.0;
                    for kc in 0..kw {
                        for kr in 0..kh {
                            let im_r = r as isize + kr as isize - pad_h;
                            let im_c = c as isize + kc as isize - pad_w;
                            if im_r >= 0 && im_r < h as isize && im_c >= 0 && im_c < w as isize {
                                let val = im.as_slice()[im_r as usize + (im_c as usize) * h];
                                let kval = kernel.as_slice()[kr + kc * kh];
                                sum += val * kval;
                            }
                        }
                    }
                    out.push(sum);
                }
            }

            let da = DenseArray::from_vec_unchecked(vec![h, w], out);
            Ok(vec![Value::Double(da)])
        }
        _ => Err(user_err(
            "MATLAB:imfilter:invalidType",
            "Image and kernel must be double matrices.",
        )),
    }
}
