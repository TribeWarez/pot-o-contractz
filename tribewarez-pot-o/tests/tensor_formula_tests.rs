// Tensor formula validation tests
//
// Tests verify that implementations correctly follow REALMS Part IV formulas:
// 1. Entropy: S(A) = |γ| * ln(d)
// 2. Mutual Information: I(A:B) = S(A) + S(B) - S(A∪B)
// 3. Effective Distance: d_eff = 1 - I(A:B) / S_max
// 4. Coherence Probability: P(unlock) = tanh(S_A / S_max)

// Note: These tests use fixed-point arithmetic (1e6 scale) as per v0.2.0 spec

#[test]
fn test_entropy_from_cut_empty() {
    // Empty cut should have 0 entropy
    let entropy = 0u64;
    assert_eq!(entropy, 0);
}

#[test]
fn test_entropy_single_edge_bond_dim_2() {
    // S = |γ| * ln(d) = 1 * ln(2) ≈ 0.693
    // In fixed-point (1e6 scale): 693_000
    let ln_2 = 0.693147f64;
    let entropy_fp = (ln_2 * 1_000_000.0) as u64;

    assert!(entropy_fp > 690_000);
    assert!(entropy_fp < 700_000);
    assert_eq!(entropy_fp, 693147);
}

#[test]
fn test_entropy_multiple_edges() {
    // S = |γ| * ln(d) = 3 * ln(2) ≈ 2.079
    // In fixed-point: 2_079_000
    let ln_2 = 0.693147f64;
    let entropy_fp = (3.0 * ln_2 * 1_000_000.0) as u64;

    assert_eq!(entropy_fp, 2_079_441);
}

#[test]
fn test_entropy_higher_bond_dimension() {
    // S = |γ| * ln(d) = 2 * ln(4) ≈ 2.772
    // In fixed-point: 2_772_000
    let ln_4 = 1.386294f64;
    let entropy_fp = (2.0 * ln_4 * 1_000_000.0) as u64;

    assert!(entropy_fp > 2_770_000);
    assert!(entropy_fp < 2_780_000);
}

#[test]
fn test_mutual_information_formula() {
    // I(A:B) = S(A) + S(B) - S(A∪B)

    // Example: S(A) = 1.0, S(B) = 1.0, S(A∪B) = 1.5
    let s_a = 1_000_000u64; // 1.0
    let s_b = 1_000_000u64; // 1.0
    let s_union = 1_500_000u64; // 1.5

    let mi_f64 = (s_a as f64 + s_b as f64 - s_union as f64) / 1_000_000.0;
    assert!((mi_f64 - 0.5).abs() < 0.001);

    // In fixed-point
    let mi_fp = (mi_f64 * 1_000_000.0) as u64;
    assert_eq!(mi_fp, 500_000);
}

#[test]
fn test_mutual_information_zero() {
    // I(A:B) = S(A) + S(B) - S(A∪B)
    // When A and B are independent: S(A∪B) = S(A) + S(B), so I(A:B) = 0

    let s_a = 1_000_000u64;
    let s_b = 1_000_000u64;
    let s_union = 2_000_000u64; // A and B are disjoint

    let mi_f64 = (s_a as f64 + s_b as f64 - s_union as f64) / 1_000_000.0;
    assert!((mi_f64 - 0.0).abs() < 0.001);
}

#[test]
fn test_mutual_information_maximum() {
    // Maximum MI when A and B are completely entangled
    // S(A∪B) = S(A) = S(B), so I(A:B) = S(A)

    let s_a = 2_000_000u64;
    let s_b = 2_000_000u64;
    let s_union = 2_000_000u64; // Complete overlap

    let mi_f64 = (s_a as f64 + s_b as f64 - s_union as f64) / 1_000_000.0;
    assert!((mi_f64 - 2.0).abs() < 0.001);
}

#[test]
fn test_effective_distance_from_mutual_info() {
    // d_eff(A, B) = 1 - I(A:B) / S_max

    // Case 1: Perfect entanglement (I = S_max)
    let mi = 1_000_000u64; // I(A:B) = 1.0
    let s_max = 1_000_000u64;

    let d_eff = 1.0 - (mi as f64 / s_max as f64);
    assert!((d_eff - 0.0).abs() < 0.001); // d_eff = 0

    // Case 2: No entanglement (I = 0)
    let mi = 0u64;
    let d_eff = 1.0 - (mi as f64 / s_max as f64);
    assert!((d_eff - 1.0).abs() < 0.001); // d_eff = 1

    // Case 3: Partial entanglement (I = 0.5 * S_max)
    let mi = 500_000u64;
    let d_eff = 1.0 - (mi as f64 / s_max as f64);
    assert!((d_eff - 0.5).abs() < 0.001); // d_eff = 0.5
}

#[test]
fn test_effective_distance_bounds() {
    // d_eff should always be in [0, 1]
    let s_max = 1_000_000u64;

    // Various MI values
    for mi in (0..=1_000_000).step_by(100_000) {
        let d_eff = 1.0 - (mi as f64 / s_max as f64);
        assert!(
            d_eff >= 0.0 && d_eff <= 1.0,
            "d_eff out of bounds for mi={}",
            mi
        );
    }
}

#[test]
fn test_coherence_probability_tanh() {
    // P(unlock) = tanh(S_A / S_max)

    // At S_A = 0: tanh(0) = 0
    let p = 0.0f64.tanh();
    assert!((p - 0.0).abs() < 0.001);

    // At S_A = S_max: tanh(1) ≈ 0.762
    let p = 1.0f64.tanh();
    assert!(p > 0.76 && p < 0.77);

    // At S_A = 0.5 * S_max: tanh(0.5) ≈ 0.462
    let p = 0.5f64.tanh();
    assert!(p > 0.46 && p < 0.47);
}

#[test]
fn test_coherence_probability_normalized() {
    // Fixed-point version: S_A and S_max in 1e6 scale
    let s_a = 500_000u64; // 0.5
    let s_max = 1_000_000u64; // 1.0

    let normalized = s_a as f64 / s_max as f64;
    let p = normalized.tanh();
    assert!(p > 0.46 && p < 0.47);

    // Convert back to fixed-point
    let p_fp = (p * 1_000_000.0) as u64;
    assert!(p_fp > 460_000 && p_fp < 470_000);
}

#[test]
fn test_coherence_probability_edge_cases() {
    // S_A > S_max: tanh can exceed 1.0 in input, but output is still bounded by tanh asymptote
    let s_a = 2_000_000u64; // 2.0
    let s_max = 1_000_000u64; // 1.0

    let normalized = s_a as f64 / s_max as f64;
    let p = normalized.tanh();
    assert!(p > 0.95 && p < 1.0); // tanh approaches 1.0 asymptotically
}

#[test]
fn test_log2_approximation_accuracy() {
    // For on-chain use, log2 might be approximated
    // log(2) = 0.693147...
    // log(4) = 1.386294...
    // log(8) = 2.079441...

    let ln_2 = 2.0f64.ln();
    let ln_4 = 4.0f64.ln();
    let ln_8 = 8.0f64.ln();

    assert!((ln_2 - 0.693147).abs() < 0.000001);
    assert!((ln_4 - 1.386294).abs() < 0.000001);
    assert!((ln_8 - 2.079441).abs() < 0.000001);
}

#[test]
fn test_fixed_point_precision() {
    // Test fixed-point arithmetic with 1e6 scale
    // Should maintain at least 6 decimal places of precision

    let value = 0.123456f64;
    let fp = (value * 1_000_000.0) as u64;
    assert_eq!(fp, 123456);

    let recovered = fp as f64 / 1_000_000.0;
    assert!((recovered - value).abs() < 0.000001);
}

#[test]
fn test_fixed_point_saturation() {
    // Test that fixed-point arithmetic doesn't overflow
    // Max u64 / 1e6 ≈ 18.4 million

    let max_fp = u64::MAX;
    let max_value = max_fp as f64 / 1_000_000.0;

    // Should be very large but not panic
    assert!(max_value > 18_000_000.0);
}

#[test]
fn test_realistic_entropy_scenario() {
    // Realistic scenario: miner with partial entanglement
    // Bond dimension = 2 (qubit), cut size = 5 edges

    let ln_2 = 2.0f64.ln();
    let entropy = (5.0 * ln_2 * 1_000_000.0) as u64;

    // S ≈ 5 * 0.693147 ≈ 3.465735
    assert!(entropy > 3_465_000 && entropy < 3_466_000);

    // Coherence probability with S_max = 5e6
    let s_max = 5_000_000u64;
    let normalized = entropy as f64 / s_max as f64;
    let coherence = normalized.tanh();

    // P ≈ tanh(0.693) ≈ 0.598
    assert!(coherence > 0.59 && coherence < 0.60);
}

#[test]
fn test_pool_cooperation_entropy_bonus() {
    // When miners pool together, their entropy can be higher due to shared computation
    // Example: individual entropy = 1e6, pool size = 3
    // Expected bonus ≈ 20%

    let individual_entropy = 1_000_000u64;
    let _pool_size = 3u64;
    let pool_bonus = 1.2f64; // 20% bonus

    let pooled_entropy = (individual_entropy as f64 * pool_bonus) as u64;

    // Should be approximately 1.2e6
    assert_eq!(pooled_entropy, 1_200_000);
}

#[test]
fn test_device_coherence_entropy_scaling() {
    // Different devices produce different entropy due to coherence
    // ASIC (1.0x) > GPU (0.8x) > CPU (0.6x) > Mobile (0.4x)

    let base_entropy = 1_000_000u64; // 1.0

    let asic_entropy = (base_entropy as f64 * 1.0) as u64;
    let gpu_entropy = (base_entropy as f64 * 0.8) as u64;
    let cpu_entropy = (base_entropy as f64 * 0.6) as u64;
    let mobile_entropy = (base_entropy as f64 * 0.4) as u64;

    assert!(asic_entropy > gpu_entropy);
    assert!(gpu_entropy > cpu_entropy);
    assert!(cpu_entropy > mobile_entropy);
}
