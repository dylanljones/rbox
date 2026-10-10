// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('../../../../.testdata/', import.meta.url))
const demo = path.join(root, 'RBv6', 'demo')
const anlz = path.join(demo, 'rekordbox', 'share', 'PIONEER', 'USBANLZ',
    'e35', 'fa187-3f34-47e2-9880-2b33cb8d1304')
const fixtures = {
    masterdb: path.join(demo, 'rekordbox', 'master.db'),
    oneLibrary: path.join(root, 'RBv7', 'demo', 'export', 'PIONEER', 'rekordbox', 'exportLibrary.db'),
    xml: path.join(demo, 'database.xml'),
    ...Object.fromEntries(['DAT', 'EXT', '2EX'].map(ext => [ext, path.join(anlz, `ANLZ0000.${ext}`)])),
}

export function temporaryPath(t, name) {
    if (!t.context.directory) {
        t.context.directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rbox-api-'))
    }
    return path.join(t.context.directory, name)
}

// Register with afterEach.always so this also runs when setup or a test fails.
export function cleanupFixtures(t) {
    delete t.context.db
    const directory = t.context.directory
    if (!directory) return
    try {
        fs.rmSync(directory, { recursive: true, force: true })
        delete t.context.directory
    } catch (error) {
        // Releasing the JS reference does not synchronously close native pools.
        // Keep the path so another cleanup attempt can retry after GC.
        if (!['EBUSY', 'EPERM', 'EACCES'].includes(error.code)) throw error
    }
}

export function fixtureCopy(t, name) {
    const source = fixtures[name]
    const destination = temporaryPath(t, path.basename(source))
    fs.copyFileSync(source, destination)
    return destination
}
