// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

/** Shared-path binding exports, number validation, and return values. */
import test from 'ava'
import { anlzDirUsb, anlzDirLocal, artworkDirLocal, artworkDirUsb, imagePathLocal } from '../index.js'

test('anlzDirUsb exports paths for ASCII and Unicode audio paths', t => {
    t.is(anlzDirUsb('/Contents/Artist/Track.flac'), '/PIONEER/USBANLZ/P02F/0000E9C7')
    t.is(anlzDirUsb('/Contents/🎵.mp3'), '/PIONEER/USBANLZ/P072/0001722E')
})

const localHelpers = [
    [anlzDirLocal, '/PIONEER/USBANLZ/0e8/0d4cc-06a6-4574-ba18-1c2301ea9924'],
    [artworkDirLocal, '/PIONEER/Artwork/0e8/0d4cc-06a6-4574-ba18-1c2301ea9924'],
    [imagePathLocal, '/PIONEER/Artwork/0e8/0d4cc-06a6-4574-ba18-1c2301ea9924/artwork.jpg'],
]

test('local path helpers normalize UUID strings', t => {
    for (const [helper, expected] of localHelpers) {
        for (const uuid of [
            '0e80d4cc-06a6-4574-ba18-1c2301ea9924',
            '0E80D4CC-06A6-4574-BA18-1C2301EA9924',
            '0e80d4cc06a64574ba181c2301ea9924',
        ]) {
            t.is(helper(uuid), expected)
        }
    }
})

test('local path helpers report invalid UUIDs as argument errors', t => {
    for (const [helper] of localHelpers) {
        for (const uuid of ['', 'not-a-uuid', '0e80d4cc-06a6-4574-ba18-1c2301ea992g']) {
            t.throws(() => helper(uuid), { code: 'InvalidArg', message: /Invalid UUID/ })
        }
    }
})

test('path helpers reject non-string arguments', t => {
    for (const helper of [anlzDirUsb, ...localHelpers.map(([helper]) => helper)]) {
        for (const value of [null, undefined, 123, true]) {
            t.throws(() => helper(value))
        }
    }
})

test('artworkDirUsb exports a string path for unsigned 32-bit IDs', t => {
    for (const [id, folder] of [
        [0, '00001'], [19, '00001'], [20, '00002'], [39, '00002'],
        [40, '00003'], [4294967295, '214748365'],
    ]) {
        t.is(artworkDirUsb(id), `/PIONEER/Artwork/${folder}`)
    }
})

test('artworkDirUsb rejects IDs that would truncate or wrap', t => {
    for (const id of [-1, 4294967296, 1.5, NaN, Infinity, -Infinity]) {
        t.throws(() => artworkDirUsb(id), { code: 'InvalidArg' })
    }
    for (const id of ['20', null, undefined, true]) {
        t.throws(() => artworkDirUsb(id))
    }
})
