// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared database errors and the database result alias.
//!
//! Distinguishes invalid connection settings, connection establishment failures,
//! query errors, and connection pool failures for database-specific handlers.

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid connection options: {0}")]
    InvalidOptions(String),

    #[error("Diesel error: {0}")]
    Query(#[from] diesel::result::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),

    #[error("Connection error: {0}")]
    Connection(#[from] diesel::ConnectionError),
}
