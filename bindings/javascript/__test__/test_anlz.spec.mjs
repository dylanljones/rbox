// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

import test from 'ava'
import { Anlz } from '../index.js'
import { cleanupFixtures, fixtureCopy, temporaryPath } from './helpers/fixtures.mjs'

test.afterEach.always(cleanupFixtures)

for (const extension of ['DAT', 'EXT', '2EX']) {
    test(`ANLZ ${extension} constructor and getters`, t => {
        const anlz = new Anlz(fixtureCopy(t, extension))
        t.true(anlz instanceof Anlz)
        const tags = anlz.getTags()
        t.true(Array.isArray(tags))
        t.true(tags.every(tag => typeof tag === 'string'))
        t.is(typeof anlz.getPath(), 'string')
        const grid = anlz.getBeatGrid()
        t.true(grid == null || typeof grid === 'object')
    })
}

test('ANLZ setter and writer signatures', t => {
    const anlz = new Anlz(fixtureCopy(t, 'DAT'))
    t.is(anlz.setPath('/api/track.mp3'), undefined)
    t.is(anlz.getPath(), '/api/track.mp3')
    t.is(anlz.dump(), undefined)
    t.is(anlz.dumpCopy(temporaryPath(t, 'copy.DAT')), undefined)
    t.throws(() => anlz.setPath({}), { instanceOf: Error })
})

test('ANLZ native errors become JavaScript errors', t => {
    t.throws(() => new Anlz(temporaryPath(t, 'missing.DAT')), { instanceOf: Error })
})
