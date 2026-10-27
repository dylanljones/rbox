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
    deviceLibrary: path.join(root, 'RBv7', 'demo', 'export', 'PIONEER', 'rekordbox', 'export.pdb'),
    deviceLibraryExtended: path.join(root, 'RBv7', 'demo', 'export', 'PIONEER', 'rekordbox', 'exportExt.pdb'),
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
export async function cleanupFixtures(t) {
    delete t.context.db
    const directory = t.context.directory
    if (!directory) return
    try {
        // Async retries let native finalizers and Windows file operations progress.
        await fs.promises.rm(directory, {
            recursive: true,
            force: true,
            maxRetries: 3,
            retryDelay: 100,
        })
        delete t.context.directory
    } catch (error) {
        // Native pools and lazy ANLZ readers can outlive their JS references.
        // Windows may report their pending deletions as a nonempty directory.
        // Keep the path so another cleanup attempt can retry after GC.
        const pendingDeletion = process.platform === 'win32' && error.code === 'ENOTEMPTY'
        if (!pendingDeletion && !['EBUSY', 'EPERM', 'EACCES'].includes(error.code)) throw error
    }
}

export function fixtureCopy(t, name, version = 6) {
    let source = fixtures[name]
    if (version !== 6) {
        if (version !== 7 || !['DAT', 'EXT', '2EX'].includes(name)) {
            throw new Error('Version override is only supported for RB7 ANLZ fixtures')
        }
        source = path.join(root, 'RBv7', 'demo', 'export', 'PIONEER', 'USBANLZ', 'P016', '0000875E', path.basename(source))
    }
    const destination = temporaryPath(t, path.basename(source))
    fs.copyFileSync(source, destination)
    return destination
}
