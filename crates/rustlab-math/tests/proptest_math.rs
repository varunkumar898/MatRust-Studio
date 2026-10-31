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
}
