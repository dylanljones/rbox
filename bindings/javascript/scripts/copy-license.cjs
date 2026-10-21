// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

// Include the workspace license in the main and native platform npm packages.

const { copyFileSync, readdirSync } = require('fs');
const { resolve, join } = require('path');

const packageRoot = resolve(__dirname, '..');
const license = resolve(packageRoot, '../../LICENSE');
const platformRoot = join(packageRoot, 'npm');

copyFileSync(license, join(packageRoot, 'LICENSE'));
for (const entry of readdirSync(platformRoot, { withFileTypes: true })) {
  if (entry.isDirectory()) {
    copyFileSync(license, join(platformRoot, entry.name, 'LICENSE'));
  }
}
