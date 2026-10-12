// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Database-independent ordering validation.
use rbox_db::validate_seq_numbers;

#[test]
fn sequence_validation_boundaries() {
    assert!(validate_seq_numbers(&[], 0));
    assert!(validate_seq_numbers(&[0, 1, 2], 0));
    assert!(validate_seq_numbers(&[1, 2, 3], 1));
    assert!(validate_seq_numbers(&[i32::MAX], i32::MAX));
    assert!(!validate_seq_numbers(&[i32::MAX, i32::MIN], i32::MAX));
    assert!(!validate_seq_numbers(&[0, 0], 0));
    assert!(!validate_seq_numbers(&[0, 2], 0));
    assert!(!validate_seq_numbers(&[2, 1], 1));
}
