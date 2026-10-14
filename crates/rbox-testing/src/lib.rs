// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared workspace fixtures and optional database test contracts.
//! Fixtures require a repository checkout containing `.testdata`.

pub mod data;

#[cfg(feature = "database")]
pub mod ordering;
