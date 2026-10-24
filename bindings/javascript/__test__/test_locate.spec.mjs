// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

/** Discovery bindings use isolated profiles and copied databases. */
import test from 'ava'
import { mkdirSync, writeFileSync, existsSync } from 'node:fs'
import path from 'node:path'
import { LookupPaths, LocationSource, MasterDb, findProgramsIn } from '../index.js'
import { cleanupFixtures, fixtureCopy, temporaryPath } from './helpers/fixtures.mjs'

test.afterEach.always(cleanupFixtures)

test('missing profiles and discovery errors', t => {
    const base = temporaryPath(t, 'profile')
    const paths = LookupPaths.under(base)
    const location = paths.locate()
    t.is(location.dbPath, path.join(base, 'Pioneer', 'rekordbox', 'master.db'))
    t.is(location.source, LocationSource.Default)
    t.false(location.available)
    t.false(existsSync(base))
    t.throws(() => MasterDb.fromLocation(location), { instanceOf: Error })
    t.false(existsSync(location.dbPath))
    paths.defaultLibraryDirs = []
    t.throws(() => paths.locate(), { instanceOf: Error })
})

test('options location opens a copied library', t => {
    const database = fixtureCopy(t, 'masterdb')
    const options = temporaryPath(t, 'options.json')
    const analysis = temporaryPath(t, 'custom-analysis')
    const settings = temporaryPath(t, 'custom-settings')
    writeFileSync(options, JSON.stringify({ options: [
        ['db-path', database], ['analysis-data-root-path', analysis],
        ['settings-root-path', settings]] }))
    const paths = new LookupPaths(temporaryPath(t, 'missing.settings'), options, [temporaryPath(t, 'fallback')])
    t.is(paths.optionsFile, options)
    paths.settingsFile = temporaryPath(t, 'other.settings')
    t.is(paths.settingsFile, temporaryPath(t, 'other.settings'))
    const location = paths.locate()
    t.is(location.source, LocationSource.Options)
    t.is(location.dbPath, database)
    t.is(location.analysisRoot, analysis)
    t.is(location.settingsRoot, settings)
    t.true(location.available)
    t.context.db = MasterDb.fromLocation(location)
    t.true(t.context.db.getContents().length > 0)
    writeFileSync(options, '{bad json')
    t.throws(() => paths.locate(), { instanceOf: Error })
})

test('program results contain paths and version components', t => {
    const base = temporaryPath(t, 'programs')
    const executable = path.join(base, 'rekordbox 7.2.1', 'rekordbox.exe')
    mkdirSync(path.dirname(executable), { recursive: true })
    writeFileSync(executable, '')
    const programs = findProgramsIn([base])
    t.is(programs.length, 1)
    t.is(programs[0].executable, executable)
    t.deepEqual(programs[0].version, [7, 2, 1])
    t.deepEqual(findProgramsIn([temporaryPath(t, 'missing')]), [])
})
