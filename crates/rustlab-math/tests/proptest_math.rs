use proptest::prelude::*;
use rustlab_math::{DenseArray, Value};

proptest! {
    #[test]
    fn prop_addition_is_commutative(
        v1 in prop::collection::vec(-1000.0f64..1000.0f64, 1..64),
        v2 in prop::collection::vec(-1000.0f64..1000.0f64, 1..64)
    ) {
        let len = v1.len().min(v2.len());
        let a = Value::Double(DenseArray::row_vector(v1[..len].to_vec()));
        let b = Value::Double(DenseArray::row_vector(v2[..len].to_vec()));

        let ab = a.add(&b).unwrap();
        let ba = b.add(&a).unwrap();

        if let (Value::Double(d_ab), Value::Double(d_ba)) = (ab, ba) {
            for (x, y) in d_ab.as_slice().iter().zip(d_ba.as_slice().iter()) {
                prop_assert!((x - y).abs() < 1e-12);
            }
        } else {
            prop_assert!(false, "Expected Double values");
        }
    }

    #[test]
    fn prop_transpose_involution(
        rows in 1usize..8,
        cols in 1usize..8,
        vals in prop::collection::vec(-500.0f64..500.0f64, 64)
    ) {
        let numel = rows * cols;
        let arr = DenseArray::from_vec(vec![rows, cols], vals[..numel].to_vec()).unwrap();
        let val = Value::Double(arr);

        let t1 = val.transpose().unwrap();
        let t2 = t1.transpose().unwrap();

        if let (Value::Double(orig), Value::Double(reconstructed)) = (val, t2) {
            prop_assert_eq!(orig.shape(), reconstructed.shape());
            prop_assert_eq!(orig.as_slice(), reconstructed.as_slice());
        } else {
            prop_assert!(false, "Expected Double values");
        }
    }

    #[test]
    fn prop_1_based_indexing_and_column_major_equivalence(
        rows in 1usize..10,
        cols in 1usize..10,
        vals in prop::collection::vec(-1000.0f64..1000.0f64, 100)
    ) {
        let numel = rows * cols;
        let arr = DenseArray::from_vec(vec![rows, cols], vals[..numel].to_vec()).unwrap();

        // 1-based indexing: 0 is out of bounds
        prop_assert!(arr.get_linear(0).is_err());
        prop_assert!(arr.get_subscript(&[0, 1]).is_err());
        prop_assert!(arr.get_subscript(&[1, 0]).is_err());

        // Past bounds is out of bounds
        prop_assert!(arr.get_linear(numel + 1).is_err());
        prop_assert!(arr.get_subscript(&[rows + 1, 1]).is_err());
        prop_assert!(arr.get_subscript(&[1, cols + 1]).is_err());

        // For every valid coordinate (r, c), check column-major index: (c - 1) * rows + r
        for r in 1..=rows {
            for c in 1..=cols {
                let expected_linear = (c - 1) * rows + r;
                let val_sub = arr.get_subscript(&[r, c]);
                let val_lin = arr.get_linear(expected_linear);
                prop_assert!(val_sub.is_ok());
                prop_assert_eq!(val_sub, val_lin);
            }
        }
    }
}
