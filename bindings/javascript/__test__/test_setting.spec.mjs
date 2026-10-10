// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

import test from 'ava'
import fs from 'node:fs'
import path from 'node:path'
import { Setting, OnAirDisplay } from '../index.js'
import { cleanupFixtures, temporaryPath } from './helpers/fixtures.mjs'

test.afterEach.always(cleanupFixtures)

for (const [factory, filename] of [
    ['newMysetting', 'MYSETTING.DAT'],
    ['newMysetting2', 'MYSETTING2.DAT'],
    ['newDjmmysetting', 'DJMMYSETTING.DAT'],
    ['newDevsetting', 'DEVSETTING.DAT'],
]) {
    test(`Setting ${factory} factory and writer signatures`, t => {
        const destination = temporaryPath(t, filename)
        const setting = Setting[factory](destination)
        t.true(setting instanceof Setting)
        t.is(setting.dump(), undefined)
        const copyDirectory = temporaryPath(t, 'copy')
        fs.mkdirSync(copyDirectory)
        t.is(setting.dumpCopy(path.join(copyDirectory, filename)), undefined)
        t.true(Setting.load(destination) instanceof Setting)
    })
}

test('settings enum conversion', t => {
    const setting = Setting.newMysetting(temporaryPath(t, 'MYSETTING.DAT'))
    t.true(Object.values(OnAirDisplay).includes(setting.getOnAirDisplay()))
    t.is(setting.setOnAirDisplay(OnAirDisplay.On), undefined)
    t.is(setting.getOnAirDisplay(), OnAirDisplay.On)
    t.throws(() => setting.setOnAirDisplay('not-an-enum-value'), { instanceOf: Error })
})

test('settings native errors become JavaScript errors', t => {
    t.throws(() => Setting.load(temporaryPath(t, 'missing.DAT')), { instanceOf: Error })
})
