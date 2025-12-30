//! Integration tests for ZKIR value types
//!
//! Tests the public API of value types including GenericValue and Value40.

use zkir_spec::{GenericValue, Value, Value40, Value40Generic, Value60, Value64, Value80};

#[test]
fn test_value_trait_polymorphism() {
    // Test that different value types work polymorphically through the Value trait
    fn compute_sum<V: Value>(a: V, b: V) -> u64 {
        a.wrapping_add(b).to_u64()
    }

    let sum40 = compute_sum(Value40::from_u64(100), Value40::from_u64(200));
    assert_eq!(sum40, 300);

    let sum60: u64 = compute_sum(
        Value60::from_u64(0x1_0000_0000),
        Value60::from_u64(0x2_0000_0000),
    );
    assert_eq!(sum60, 0x3_0000_0000);
}

#[test]
fn test_value_overflow_behavior() {
    // 40-bit overflow
    let max40 = Value40::from_u64((1u64 << 40) - 1);
    let one = Value40::from_u64(1);
    let wrapped = max40.wrapping_add(one);
    assert_eq!(wrapped.to_u64(), 0);

    // 60-bit overflow
    let max60: Value60 = GenericValue::from_u128((1u128 << 60) - 1);
    let one60: Value60 = GenericValue::from_u64(1);
    let wrapped60 = max60.wrapping_add(one60);
    assert_eq!(wrapped60.to_u128(), 0);
}

#[test]
fn test_value_underflow_behavior() {
    let zero = Value40::from_u64(0);
    let one = Value40::from_u64(1);
    let wrapped = zero.wrapping_sub(one);
    assert_eq!(wrapped.to_u64(), (1u64 << 40) - 1);
}

#[test]
fn test_cross_limb_arithmetic() {
    // Test arithmetic that crosses limb boundaries
    // With 20-bit limbs, 0xFFFFF is max for one limb
    let a = Value40::from_u64(0xFFFFF); // Max first limb
    let b = Value40::from_u64(1);
    let sum = a.wrapping_add(b);
    assert_eq!(sum.to_u64(), 0x100000); // Should carry into second limb
    assert_eq!(sum.limbs()[0], 0); // First limb wraps to 0
    assert_eq!(sum.limbs()[1], 1); // Second limb gets carry
}

#[test]
fn test_multiplication_overflow() {
    // Large multiplication that overflows
    let a = Value40::from_u64(0x100000); // 2^20
    let b = Value40::from_u64(0x100000); // 2^20
    let product = a.wrapping_mul(b); // 2^40, should wrap to 0
    assert_eq!(product.to_u64(), 0);

    // Just under overflow
    let c = Value40::from_u64(0xFFFFF); // 2^20 - 1
    let d = Value40::from_u64(2);
    let product2 = c.wrapping_mul(d);
    assert_eq!(product2.to_u64(), 0x1FFFFE);
}

#[test]
fn test_bitwise_operations_comprehensive() {
    let all_ones = Value40::from_u64((1u64 << 40) - 1);
    let zeros = Value40::from_u64(0);

    // NOT of zeros should be all ones (within 40 bits)
    let not_zeros = zeros.bitwise_not();
    assert_eq!(not_zeros.to_u64(), (1u64 << 40) - 1);

    // NOT of all ones should be zero
    let not_ones = all_ones.bitwise_not();
    assert_eq!(not_ones.to_u64(), 0);

    // XOR with self should be zero
    let xor_self = all_ones.bitwise_xor(all_ones);
    assert_eq!(xor_self.to_u64(), 0);

    // AND with zeros should be zero
    let and_zeros = all_ones.bitwise_and(zeros);
    assert_eq!(and_zeros.to_u64(), 0);

    // OR with zeros should be unchanged
    let or_zeros = all_ones.bitwise_or(zeros);
    assert_eq!(or_zeros.to_u64(), all_ones.to_u64());
}

#[test]
fn test_shift_edge_cases() {
    let v = Value40::from_u64(0x8000000001); // Bit 39 and bit 0 set

    // Shift by 0 should be unchanged
    assert_eq!(v.left_shift(0).to_u64(), v.to_u64());
    assert_eq!(v.right_shift(0).to_u64(), v.to_u64());

    // Shift by total bits should be zero
    assert_eq!(v.left_shift(40).to_u64(), 0);
    assert_eq!(v.right_shift(40).to_u64(), 0);

    // Shift by more than total bits should be zero
    assert_eq!(v.left_shift(100).to_u64(), 0);
    assert_eq!(v.right_shift(100).to_u64(), 0);
}

#[test]
fn test_arithmetic_right_shift() {
    // Positive number (no sign extension)
    let pos = Value40::from_u64(0x7FFFFFFF); // 31 bits, positive in 32-bit
    let shifted = pos.arithmetic_right_shift(4, 32);
    assert_eq!(shifted.to_u64(), 0x07FFFFFF);

    // Negative number (with sign extension)
    let neg = Value40::from_u64(0x80000000); // -2^31 in 32-bit
    let shifted_neg = neg.arithmetic_right_shift(4, 32);
    // Should have sign bits filled in from the left
    assert_eq!(shifted_neg.to_u64() & 0xFFFFFFFF, 0xF8000000);
}

#[test]
fn test_sign_extension_comprehensive() {
    // 8-bit to 40-bit
    let byte_neg = Value40::from_u64(0x80); // -128 in 8-bit
    let extended = byte_neg.sign_extend(8, 40);
    assert_eq!(extended.to_u64(), 0xFFFFFFFF80);

    let byte_pos = Value40::from_u64(0x7F); // +127 in 8-bit
    let extended_pos = byte_pos.sign_extend(8, 40);
    assert_eq!(extended_pos.to_u64(), 0x7F);

    // 16-bit to 40-bit
    let half_neg = Value40::from_u64(0x8000); // -32768 in 16-bit
    let extended16 = half_neg.sign_extend(16, 40);
    assert_eq!(extended16.to_u64(), 0xFFFFFF8000);
}

#[test]
fn test_truncation() {
    let full = Value40::from_u64(0xFFFFFFFFFF); // All 40 bits set

    assert_eq!(full.truncate(8).to_u64(), 0xFF);
    assert_eq!(full.truncate(16).to_u64(), 0xFFFF);
    assert_eq!(full.truncate(32).to_u64(), 0xFFFFFFFF);
    assert_eq!(full.truncate(40).to_u64(), 0xFFFFFFFFFF);
}

#[test]
fn test_fits_in() {
    let small = Value40::from_u64(0xFF);
    assert!(small.fits_in(8));
    assert!(small.fits_in(16));

    let medium = Value40::from_u64(0x100);
    assert!(!medium.fits_in(8));
    assert!(medium.fits_in(9));
    assert!(medium.fits_in(16));

    let large = Value40::from_u64(0x1_0000_0000);
    assert!(!large.fits_in(32));
    assert!(large.fits_in(33));
}

#[test]
fn test_signed_comparison() {
    // In 32-bit signed interpretation
    let pos = Value40::from_u64(100);
    let neg = Value40::from_u64(0xFFFFFFFF); // -1 in 32-bit signed
    let zero = Value40::from_u64(0);

    // -1 < 0
    assert!(neg.signed_lt(zero, 32));
    // -1 < 100
    assert!(neg.signed_lt(pos, 32));
    // 0 < 100
    assert!(zero.signed_lt(pos, 32));
    // 100 not < 0
    assert!(!pos.signed_lt(zero, 32));
}

#[test]
fn test_value64_full_range() {
    // Value64 should handle full u64 range
    let max: Value64 = GenericValue::from_u64(u64::MAX);
    assert_eq!(max.to_u64(), u64::MAX);

    let min: Value64 = GenericValue::from_u64(0);
    assert_eq!(min.to_u64(), 0);

    // Arithmetic should work correctly
    let half: Value64 = GenericValue::from_u64(u64::MAX / 2);
    let sum = half.wrapping_add(half);
    assert_eq!(sum.to_u64(), u64::MAX - 1);
}

#[test]
fn test_value80_beyond_u64() {
    // Value80 can hold values beyond u64::MAX
    let beyond: Value80 = GenericValue::from_u128(u64::MAX as u128 + 1);
    assert_eq!(beyond.to_u128(), 0x1_0000_0000_0000_0000);

    // But to_u64 truncates
    assert_eq!(beyond.to_u64(), 0);

    // Large arithmetic
    let a: Value80 = GenericValue::from_u128(0x1_0000_0000_0000_0000);
    let b: Value80 = GenericValue::from_u128(0x1_0000_0000_0000_0000);
    let sum = a.wrapping_add(b);
    assert_eq!(sum.to_u128(), 0x2_0000_0000_0000_0000);
}

#[test]
fn test_from_limbs_validation() {
    // Values larger than LIMB_MASK should be masked
    let limbs = [0xFFFFFFFF, 0xFFFFFFFF]; // Both exceed 20-bit mask
    let v = Value40::from_limbs(&limbs);

    // Should be masked to 20 bits each
    assert_eq!(v.limbs()[0], 0xFFFFF);
    assert_eq!(v.limbs()[1], 0xFFFFF);
    assert_eq!(v.to_u64(), (1u64 << 40) - 1);
}

#[test]
fn test_generic_value_type_aliases_consistency() {
    // Verify type aliases have expected properties
    assert_eq!(Value40Generic::TOTAL_BITS, 40);
    assert_eq!(Value60::TOTAL_BITS, 60);
    assert_eq!(Value80::TOTAL_BITS, 80);
    assert_eq!(Value64::TOTAL_BITS, 64);

    // All should have correct limb counts
    assert_eq!(<Value40Generic as Value>::NUM_LIMBS, 2);
    assert_eq!(<Value60 as Value>::NUM_LIMBS, 3);
    assert_eq!(<Value80 as Value>::NUM_LIMBS, 4);
    assert_eq!(<Value64 as Value>::NUM_LIMBS, 2);
}
