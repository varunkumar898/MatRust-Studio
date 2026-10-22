use matlab_runtime::run_source;
use matlab_values::Value;

// ============================================================================
// 1. Base conversion functions: dec2bin, bin2dec, dec2hex, hex2dec
// ============================================================================

#[test]
fn dec2bin_basic() {
    let r = run_source("s1 = dec2bin(0); s2 = dec2bin(1); s3 = dec2bin(10); s4 = dec2bin(255)")
        .unwrap();
    assert_eq!(r.env.get("s1").unwrap(), &Value::String("0".into()));
    assert_eq!(r.env.get("s2").unwrap(), &Value::String("1".into()));
    assert_eq!(r.env.get("s3").unwrap(), &Value::String("1010".into()));
    assert_eq!(r.env.get("s4").unwrap(), &Value::String("11111111".into()));
}

#[test]
fn dec2bin_with_width() {
    let r = run_source("s1 = dec2bin(10, 8); s2 = dec2bin(255, 4)").unwrap();
    // width 8 pads with leading zeros
    assert_eq!(r.env.get("s1").unwrap(), &Value::String("00001010".into()));
    // width 4 is smaller than 8 bits, so it does not truncate
    assert_eq!(r.env.get("s2").unwrap(), &Value::String("11111111".into()));
}

#[test]
fn dec2bin_negative_twos_complement() {
    let r = run_source("s = dec2bin(-1)").unwrap();
    // 32-bit two's complement
    assert_eq!(
        r.env.get("s").unwrap(),
        &Value::String("11111111111111111111111111111111".into())
    );
}

#[test]
fn dec2bin_invalid_inputs() {
    assert!(run_source("dec2bin(10.5)").is_err());
    assert!(run_source("dec2bin()").is_err());
    assert!(run_source("dec2bin(1, 2, 3)").is_err());
}

#[test]
fn bin2dec_basic() {
    let r = run_source(
        "n1 = bin2dec('0'); n2 = bin2dec('1'); n3 = bin2dec('1010'); n4 = bin2dec('11111111')",
    )
    .unwrap();
    assert_eq!(r.env.get("n1").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(r.env.get("n2").unwrap().as_matrix().unwrap().data[0], 1.0);
    assert_eq!(r.env.get("n3").unwrap().as_matrix().unwrap().data[0], 10.0);
    assert_eq!(r.env.get("n4").unwrap().as_matrix().unwrap().data[0], 255.0);
}

#[test]
fn bin2dec_padded_and_trimmed() {
    let r = run_source("n1 = bin2dec('00001010'); n2 = bin2dec('  1010  ')").unwrap();
    assert_eq!(r.env.get("n1").unwrap().as_matrix().unwrap().data[0], 10.0);
    assert_eq!(r.env.get("n2").unwrap().as_matrix().unwrap().data[0], 10.0);
}

#[test]
fn bin2dec_invalid_inputs() {
    assert!(run_source("bin2dec('102')").is_err());
    assert!(run_source("bin2dec('abc')").is_err());
    assert!(run_source("bin2dec('')").is_err());
    assert!(run_source("bin2dec()").is_err());
    assert!(run_source("bin2dec('1', '2')").is_err());
}

#[test]
fn roundtrip_dec2bin_bin2dec() {
    let script = "
        v = [0, 1, 2, 7, 10, 42, 255, 1024, 65535, 1000000];
        ok = 1;
        for x = v
            b = dec2bin(x);
            d = bin2dec(b);
            if d ~= x
                ok = 0;
            end
        end
    ";
    let r = run_source(script).unwrap();
    assert_eq!(r.env.get("ok").unwrap().as_matrix().unwrap().data[0], 1.0);
}

#[test]
fn dec2hex_basic() {
    let r = run_source("h1 = dec2hex(0); h2 = dec2hex(10); h3 = dec2hex(255); h4 = dec2hex(4095)")
        .unwrap();
    assert_eq!(r.env.get("h1").unwrap(), &Value::String("0".into()));
    assert_eq!(r.env.get("h2").unwrap(), &Value::String("A".into()));
    assert_eq!(r.env.get("h3").unwrap(), &Value::String("FF".into()));
    assert_eq!(r.env.get("h4").unwrap(), &Value::String("FFF".into()));
}

#[test]
fn dec2hex_with_width() {
    let r = run_source("h1 = dec2hex(255, 4); h2 = dec2hex(255, 1)").unwrap();
    assert_eq!(r.env.get("h1").unwrap(), &Value::String("00FF".into()));
    assert_eq!(r.env.get("h2").unwrap(), &Value::String("FF".into()));
}

#[test]
fn dec2hex_negative_twos_complement() {
    let r = run_source("h = dec2hex(-1)").unwrap();
    assert_eq!(r.env.get("h").unwrap(), &Value::String("FFFFFFFF".into()));
}

#[test]
fn dec2hex_invalid_inputs() {
    assert!(run_source("dec2hex(10.5)").is_err());
    assert!(run_source("dec2hex()").is_err());
}

#[test]
fn hex2dec_basic() {
    let r = run_source("n1 = hex2dec('0'); n2 = hex2dec('A'); n3 = hex2dec('a'); n4 = hex2dec('FF'); n5 = hex2dec('0x10'); n6 = hex2dec('0XFF')").unwrap();
    assert_eq!(r.env.get("n1").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(r.env.get("n2").unwrap().as_matrix().unwrap().data[0], 10.0);
    assert_eq!(r.env.get("n3").unwrap().as_matrix().unwrap().data[0], 10.0);
    assert_eq!(r.env.get("n4").unwrap().as_matrix().unwrap().data[0], 255.0);
    assert_eq!(r.env.get("n5").unwrap().as_matrix().unwrap().data[0], 16.0);
    assert_eq!(r.env.get("n6").unwrap().as_matrix().unwrap().data[0], 255.0);
}

#[test]
fn hex2dec_invalid_inputs() {
    assert!(run_source("hex2dec('1G')").is_err());
    assert!(run_source("hex2dec('')").is_err());
    assert!(run_source("hex2dec()").is_err());
}

#[test]
fn roundtrip_dec2hex_hex2dec() {
    let script = "
        v = [0, 1, 15, 16, 255, 4096, 65535, 1000000];
        ok = 1;
        for x = v
            h = dec2hex(x);
            d = hex2dec(h);
            if d ~= x
                ok = 0;
            end
        end
    ";
    let r = run_source(script).unwrap();
    assert_eq!(r.env.get("ok").unwrap().as_matrix().unwrap().data[0], 1.0);
}

// ============================================================================
// 2. Inverse trigonometry in degrees: asind, acosd, atand
// ============================================================================

#[test]
fn asind_domain_and_boundaries() {
    let r = run_source(
        "a0 = asind(0); a1 = asind(1); am1 = asind(-1); ahalf = asind(0.5); aout = asind(2)",
    )
    .unwrap();
    assert_eq!(r.env.get("a0").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(r.env.get("a1").unwrap().as_matrix().unwrap().data[0], 90.0);
    assert_eq!(
        r.env.get("am1").unwrap().as_matrix().unwrap().data[0],
        -90.0
    );
    let ahalf = r.env.get("ahalf").unwrap().as_matrix().unwrap().data[0];
    assert!((ahalf - 30.0).abs() < 1e-10);
    assert!(r.env.get("aout").unwrap().as_matrix().unwrap().data[0].is_nan());
}

#[test]
fn acosd_domain_and_boundaries() {
    let r = run_source(
        "c1 = acosd(1); c0 = acosd(0); cm1 = acosd(-1); chalf = acosd(0.5); cout = acosd(2)",
    )
    .unwrap();
    assert_eq!(r.env.get("c1").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(r.env.get("c0").unwrap().as_matrix().unwrap().data[0], 90.0);
    assert_eq!(
        r.env.get("cm1").unwrap().as_matrix().unwrap().data[0],
        180.0
    );
    let chalf = r.env.get("chalf").unwrap().as_matrix().unwrap().data[0];
    assert!((chalf - 60.0).abs() < 1e-10);
    assert!(r.env.get("cout").unwrap().as_matrix().unwrap().data[0].is_nan());
}

#[test]
fn atand_values_and_infinities() {
    let r = run_source(
        "t0 = atand(0); t1 = atand(1); tm1 = atand(-1); tinf = atand(1./0); tminf = atand(-1./0)",
    )
    .unwrap();
    assert_eq!(r.env.get("t0").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(r.env.get("t1").unwrap().as_matrix().unwrap().data[0], 45.0);
    assert_eq!(
        r.env.get("tm1").unwrap().as_matrix().unwrap().data[0],
        -45.0
    );
    assert_eq!(
        r.env.get("tinf").unwrap().as_matrix().unwrap().data[0],
        90.0
    );
    assert_eq!(
        r.env.get("tminf").unwrap().as_matrix().unwrap().data[0],
        -90.0
    );
}

#[test]
fn degree_inverse_trig_arrays() {
    let r = run_source("A = asind([0, 1; -1, 0.5])").unwrap();
    let m = r.env.get("A").unwrap().as_matrix().unwrap();
    assert_eq!(m.rows, 2);
    assert_eq!(m.cols, 2);
    assert_eq!(m.data[0], 0.0);
    assert_eq!(m.data[1], 90.0);
    assert_eq!(m.data[2], -90.0);
    assert!((m.data[3] - 30.0).abs() < 1e-10);
}

// ============================================================================
// 3. Prime factorization: factor
// ============================================================================

#[test]
fn factor_smallest_and_primes() {
    let r = run_source("f1 = factor(1); f2 = factor(2); f13 = factor(13)").unwrap();
    let m1 = r.env.get("f1").unwrap().as_matrix().unwrap();
    assert_eq!(m1.rows, 1);
    assert_eq!(m1.cols, 1);
    assert_eq!(m1.data, vec![1.0]);

    let m2 = r.env.get("f2").unwrap().as_matrix().unwrap();
    assert_eq!(m2.data, vec![2.0]);

    let m13 = r.env.get("f13").unwrap().as_matrix().unwrap();
    assert_eq!(m13.data, vec![13.0]);
}

#[test]
fn factor_composites_and_repeated_factors() {
    let r = run_source("f4 = factor(4); f200 = factor(200); f1024 = factor(1024)").unwrap();
    let m4 = r.env.get("f4").unwrap().as_matrix().unwrap();
    assert_eq!(m4.data, vec![2.0, 2.0]);

    let m200 = r.env.get("f200").unwrap().as_matrix().unwrap();
    assert_eq!(m200.data, vec![2.0, 2.0, 2.0, 5.0, 5.0]);

    let m1024 = r.env.get("f1024").unwrap().as_matrix().unwrap();
    assert_eq!(m1024.data, vec![2.0; 10]);
}

#[test]
fn factor_product_reconstruction() {
    let script = "
        numbers = [1, 2, 6, 12, 100, 360, 1001, 65535];
        ok = 1;
        for n = numbers
            f = factor(n);
            prod_val = 1;
            for p = f
                prod_val = prod_val * p;
            end
            if prod_val ~= n
                ok = 0;
            end
        end
    ";
    let r = run_source(script).unwrap();
    assert_eq!(r.env.get("ok").unwrap().as_matrix().unwrap().data[0], 1.0);
}

#[test]
fn factor_invalid_inputs() {
    assert!(run_source("factor(0)").is_err());
    assert!(run_source("factor(-5)").is_err());
    assert!(run_source("factor(2.5)").is_err());
    assert!(run_source("factor([2, 3])").is_err());
    assert!(run_source("factor()").is_err());
}

// ============================================================================
// 4. Edge cases & Batch 6 fixes: nthroot, mod/rem, exact degree trig, overflow
// ============================================================================

#[test]
fn nthroot_negative_real_inputs() {
    let r = run_source(
        "r1 = nthroot(-8, 3); r2 = nthroot(-27, 3); r3 = nthroot(-32, 5); r4 = nthroot(8, 3)",
    )
    .unwrap();
    assert_eq!(r.env.get("r1").unwrap().as_matrix().unwrap().data[0], -2.0);
    assert_eq!(r.env.get("r2").unwrap().as_matrix().unwrap().data[0], -3.0);
    assert_eq!(r.env.get("r3").unwrap().as_matrix().unwrap().data[0], -2.0);
    assert_eq!(r.env.get("r4").unwrap().as_matrix().unwrap().data[0], 2.0);
    // Negative number with even root must error
    assert!(run_source("nthroot(-4, 2)").is_err());
}

#[test]
fn nthroot_scalar_expansion() {
    let r = run_source("R = nthroot([-8, 8, -27], 3)").unwrap();
    let m = r.env.get("R").unwrap().as_matrix().unwrap();
    assert_eq!(m.data, vec![-2.0, 2.0, -3.0]);
}

#[test]
fn mod_vs_rem_division_by_zero_and_signs() {
    let r = run_source(
        "
        m0 = mod(5, 0);
        r0 = rem(5, 0);
        m_neg = mod(-5, 3);
        r_neg = rem(-5, 3);
        m_neg_div = mod(5, -3);
        r_neg_div = rem(5, -3);
    ",
    )
    .unwrap();
    // mod(5, 0) == 5, rem(5, 0) == NaN
    assert_eq!(r.env.get("m0").unwrap().as_matrix().unwrap().data[0], 5.0);
    assert!(r.env.get("r0").unwrap().as_matrix().unwrap().data[0].is_nan());
    // mod has sign of divisor (3 > 0 -> 1)
    assert_eq!(
        r.env.get("m_neg").unwrap().as_matrix().unwrap().data[0],
        1.0
    );
    // rem has sign of dividend (-5 < 0 -> -2)
    assert_eq!(
        r.env.get("r_neg").unwrap().as_matrix().unwrap().data[0],
        -2.0
    );
    // mod has sign of divisor (-3 < 0 -> -1)
    assert_eq!(
        r.env.get("m_neg_div").unwrap().as_matrix().unwrap().data[0],
        -1.0
    );
    // rem has sign of dividend (5 > 0 -> 2)
    assert_eq!(
        r.env.get("r_neg_div").unwrap().as_matrix().unwrap().data[0],
        2.0
    );
}

#[test]
fn exact_degree_trig() {
    let r = run_source(
        "
        s0 = sind(0); s90 = sind(90); s180 = sind(180); s270 = sind(270); s360 = sind(360);
        c0 = cosd(0); c90 = cosd(90); c180 = cosd(180); c270 = cosd(270); c360 = cosd(360);
        t0 = tand(0); t45 = tand(45); t180 = tand(180);
    ",
    )
    .unwrap();
    assert_eq!(r.env.get("s0").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(r.env.get("s90").unwrap().as_matrix().unwrap().data[0], 1.0);
    assert_eq!(r.env.get("s180").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(
        r.env.get("s270").unwrap().as_matrix().unwrap().data[0],
        -1.0
    );
    assert_eq!(r.env.get("s360").unwrap().as_matrix().unwrap().data[0], 0.0);

    assert_eq!(r.env.get("c0").unwrap().as_matrix().unwrap().data[0], 1.0);
    assert_eq!(r.env.get("c90").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(
        r.env.get("c180").unwrap().as_matrix().unwrap().data[0],
        -1.0
    );
    assert_eq!(r.env.get("c270").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert_eq!(r.env.get("c360").unwrap().as_matrix().unwrap().data[0], 1.0);

    assert_eq!(r.env.get("t0").unwrap().as_matrix().unwrap().data[0], 0.0);
    assert!((r.env.get("t45").unwrap().as_matrix().unwrap().data[0] - 1.0).abs() < 1e-12);
    assert_eq!(r.env.get("t180").unwrap().as_matrix().unwrap().data[0], 0.0);
}

#[test]
fn atan2_argument_ordering_and_expansion() {
    let r = run_source(
        "
        a1 = atan2(1, 0);
        a2 = atan2(0, 1);
        A = atan2([1, -1], 0);
    ",
    )
    .unwrap();
    let pi_2 = std::f64::consts::FRAC_PI_2;
    assert!((r.env.get("a1").unwrap().as_matrix().unwrap().data[0] - pi_2).abs() < 1e-12);
    assert_eq!(r.env.get("a2").unwrap().as_matrix().unwrap().data[0], 0.0);
    let a_mat = r.env.get("A").unwrap().as_matrix().unwrap();
    assert!((a_mat.data[0] - pi_2).abs() < 1e-12);
    assert!((a_mat.data[1] - (-pi_2)).abs() < 1e-12);
}

#[test]
fn factorial_large_and_overflow() {
    let r = run_source(
        "f0 = factorial(0); f5 = factorial(5); f21 = factorial(21); f171 = factorial(171)",
    )
    .unwrap();
    assert_eq!(r.env.get("f0").unwrap().as_matrix().unwrap().data[0], 1.0);
    assert_eq!(r.env.get("f5").unwrap().as_matrix().unwrap().data[0], 120.0);
    let f21 = r.env.get("f21").unwrap().as_matrix().unwrap().data[0];
    assert!(f21 > 5e19); // Does not saturate at u64::MAX ~ 1.8e19
    assert_eq!(
        r.env.get("f171").unwrap().as_matrix().unwrap().data[0],
        f64::INFINITY
    );
}

#[test]
fn nchoosek_large_no_early_overflow() {
    let r = run_source(
        "c1 = nchoosek(5, 2); c2 = nchoosek(100, 98); c3 = nchoosek(5, 0); c4 = nchoosek(5, 5)",
    )
    .unwrap();
    assert_eq!(r.env.get("c1").unwrap().as_matrix().unwrap().data[0], 10.0);
    assert_eq!(
        r.env.get("c2").unwrap().as_matrix().unwrap().data[0],
        4950.0
    );
    assert_eq!(r.env.get("c3").unwrap().as_matrix().unwrap().data[0], 1.0);
    assert_eq!(r.env.get("c4").unwrap().as_matrix().unwrap().data[0], 1.0);
    assert!(run_source("nchoosek(5, 6)").is_err());
}

#[test]
fn bitshift_safe_shift_and_masking() {
    let r = run_source(
        "
        s1 = bitshift(1, 2);
        s2 = bitshift(16, -2);
        s_overflow = bitshift(1, 64);
        s_huge = bitshift(1, 100);
        s_neg_huge = bitshift(16, -64);
        s_masked = bitshift(13, 2, 4);
    ",
    )
    .unwrap();
    assert_eq!(r.env.get("s1").unwrap().as_matrix().unwrap().data[0], 4.0);
    assert_eq!(r.env.get("s2").unwrap().as_matrix().unwrap().data[0], 4.0);
    assert_eq!(
        r.env.get("s_overflow").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert_eq!(
        r.env.get("s_huge").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert_eq!(
        r.env.get("s_neg_huge").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert_eq!(
        r.env.get("s_masked").unwrap().as_matrix().unwrap().data[0],
        4.0
    );
}

#[test]
fn bitwise_scalar_expansion_and_validation() {
    let r = run_source("B = bitand([1, 2, 3], 1)").unwrap();
    assert_eq!(
        r.env.get("B").unwrap().as_matrix().unwrap().data,
        vec![1.0, 0.0, 1.0]
    );
    assert!(run_source("bitand(1.5, 2)").is_err());
}

#[test]
fn primes_and_isprime() {
    let r = run_source("p = primes(10); ip = isprime([1, 2, 3, 4, 5])").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert_eq!(p.rows, 1);
    assert_eq!(p.cols, 4);
    assert_eq!(p.data, vec![2.0, 3.0, 5.0, 7.0]);

    let ip = r.env.get("ip").unwrap().as_matrix().unwrap();
    assert_eq!(ip.data, vec![0.0, 1.0, 1.0, 0.0, 1.0]);

    let r_empty = run_source("p0 = primes(1)").unwrap();
    let p0 = r_empty.env.get("p0").unwrap().as_matrix().unwrap();
    assert_eq!(p0.rows, 1);
    assert_eq!(p0.cols, 0);
}
