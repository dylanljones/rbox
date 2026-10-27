// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

// Configure binding tests and native SQLCipher logging before workers start.
// Native Rust reads the OS environment, so set this in AVA's main process.
process.env.RBOX_SQLCIPHER_LOG_LEVEL ??= 'ERROR'

export default {
    concurrency: 1,
    timeout: '3m',
    files: ['./__test__/*.mjs'],
}
