// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

import test from 'ava'
import fs from 'node:fs'
import { Anlz } from '../index.js'
import { cleanupFixtures, fixtureCopy, temporaryPath } from './helpers/fixtures.mjs'


for (const [extension, version] of ['DAT', 'EXT', '2EX'].flatMap(ext => [6, 7].map(v => [ext, v]))) {
    test(`ANLZ RB${version} ${extension} typed setters and removers`, t => {
        const source = fixtureCopy(t, extension, version)
        const original = fs.readFileSync(source)
        const anlz = new Anlz(source)
        const methods = {"PQTZ":"BeatGrid","PQT2":"ExtendedBeatGrid","PCOB":"CueList","PCO2":"ExtendedCueList","PVBR":"Vbr","PVB2":"SeekIndex","PWV2":"TinyWaveformPreview","PWAV":"WaveformPreview","PWV3":"WaveformDetail","PWV4":"WaveformColorPreview","PWV5":"WaveformColorDetail","PWV6":"Waveform3BandPreview","PWV7":"Waveform3BandDetail","PWVC":"Waveform3BandCalibration","PSSI":"SongStructure","PVDI":"VocalDetection"}
        for (const section of anlz.getData().sections) {
            const suffix = methods[section.tag]
            if (suffix) anlz['set' + suffix](section.content)
        }
        const output = temporaryPath(t, 'typed.' + extension)
        anlz.dumpCopy(output)
        t.deepEqual(fs.readFileSync(output), original)
        for (const suffix of Object.values(methods)) {
            while (anlz['remove' + suffix]()) { /* remove duplicates */ }
            t.false(anlz['remove' + suffix]())
        }
    })
}


test('ANLZ preserves bigint seek values and song structure flags', t => {
    const path = temporaryPath(t, 'metadata.EXT')
    const anlz = Anlz.newExt(path)
    const sample = 2n ** 60n + 1n
    anlz.setSeekIndex({ u1: 0, totalSamples: sample, entries: [{ sample, offset: 2n ** 64n - 1n, frameSamples: 1024 }] })
    t.is(anlz.getSeekIndex().entries[0].sample, sample)
    t.throws(() => anlz.setSeekIndex({ u1: 0, totalSamples: -1n, entries: [] }))
    t.throws(() => anlz.setSeekIndex({ u1: 0, totalSamples: 2n ** 64n, entries: [] }))
    t.is(anlz.getSeekIndex().totalSamples, sample)
    t.throws(() => anlz.setVbr({ u1: 0, index: Array(399).fill(0), u2: 0 }))
    t.true(anlz.getVbr() == null)
    const phrase = {
        index: 1, beat: 1, kind: 1, k1: 0, k2: 0, b: 0, beat2: 0, beat3: 0,
        beat4: 0, k3: 0, fill: 0, beatFill: 0, u1: 7, u2: 8, u3: 9, u4: 10, u5: 11,
    }
    const data = { mood: 2, endBeat: 8, bank: 0, phrases: [phrase], u1: 12, u2: 13, u3: 14, u4: 15 }
    anlz.setSongStructure({ version: 1, isEncrypted: true, data })
    data.endBeat = 16
    t.is(anlz.getSongStructure().endBeat, 8)
    anlz.setSongStructureData(data)
    const snapshot = anlz.getSongStructureSection()
    t.is(snapshot.version, 1)
    t.true(snapshot.isEncrypted)
    t.is(snapshot.data.endBeat, 16)
    t.is(snapshot.data.phrases[0].u5, 11)
    anlz.dump()
    const reloaded = new Anlz(path)
    t.is(reloaded.getSongStructureSection().data.phrases[0].u1, 7)
    t.is(reloaded.getSeekIndex().entries[0].offset, 2n ** 64n - 1n)
    t.deepEqual(anlz.intoData().header.code, [80, 77, 65, 73])
    t.is(anlz.sectionCount(), 2)
})

test('ANLZ cue subtype helpers preserve independent lists', t => {
    const anlz = new Anlz(fixtureCopy(t, 'EXT'))
    for (const suffix of ['HotCues', 'MemoryCues', 'ExtendedHotCues', 'ExtendedMemoryCues']) {
        const value = anlz['get' + suffix]()
        t.truthy(value)
        t.true(anlz['remove' + suffix]())
        t.false(anlz['remove' + suffix]())
        anlz['set' + suffix](value)
        t.is(anlz['get' + suffix]().listType, value.listType)
    }
})

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

test('ANLZ extended grid exposes precision, endpoints and checksum', t => {
    const dat = new Anlz(fixtureCopy(t, 'DAT'))
    const ext = new Anlz(fixtureCopy(t, 'EXT'))
    const beats = dat.getBeatGrid().beats
    const grid = ext.getExtendedBeatGrid()
    t.is(grid.firstBeat.time, beats[0].time)
    t.is(grid.lastBeat.time, beats.at(-1).time)
    t.is(grid.checksum, beats.reduce((sum, b) => sum + b.beatNumber + b.tempo + b.time, 0) % 2 ** 32)
    t.is(grid.offsetMs, 0)
    t.is(grid.offsetFlags, 0)
    t.is(grid.beats.length, beats.length)
    t.true(grid.beats.every(b => b.microseconds >= 0 && b.microseconds <= 999))
})

for (const [extension, version] of ['DAT', 'EXT', '2EX'].flatMap(ext => [6, 7].map(v => [ext, v]))) {
    test(`ANLZ RB${version} ${extension} editable snapshots preserve bytes`, t => {
        const source = fixtureCopy(t, extension, version)
        const original = fs.readFileSync(source)
        const anlz = new Anlz(source)
        const data = anlz.getData()
        t.is(data.sections.length, anlz.sectionCount())
        for (const [index, section] of data.sections.entries()) {
            t.true(anlz.contains(section.tag))
            t.is(anlz.getSection(index).tag, section.tag)
            t.true(anlz.getSectionsByTag(section.tag).length > 0)
            anlz.setSectionAt(index, section)
        }
        const output = temporaryPath(t, 'snapshot.' + extension)
        anlz.dumpCopy(output)
        t.deepEqual(fs.readFileSync(output), original)
        t.deepEqual(fs.readFileSync(source), original)
    })
}

for (const [extension, factory] of [['DAT', 'newDat'], ['EXT', 'newExt'], ['2EX', 'new2ex']]) {
    test(`ANLZ ${extension} creation and explicit setters`, t => {
        const path = temporaryPath(t, 'created.' + extension)
        const anlz = Anlz[factory](path)
        t.false(fs.existsSync(path))
        t.is(anlz.sectionCount(), 0)
        t.true(anlz.getPath() == null)
        t.true(anlz.getSection(0) == null)
        anlz.setPath('/music/test.aiff')
        const grid = { beats: [{ beatNumber: 1, tempo: 12000, time: 100 }], headerFlags: 0x80000 }
        anlz.setBeatGrid(grid)
        grid.beats[0].time = 999
        t.is(anlz.getBeatGrid().beats[0].time, 100)
        anlz.setBeatGrid(grid)
        t.is(anlz.getBeatGrid().beats[0].time, 999)
        const unknown = { tag: 'Unknown', content: { code: [90, 90, 90, 90], headerData: [1, 2], contentData: [3, 4] } }
        anlz.pushSection(unknown)
        anlz.insertSection(1, unknown)
        t.is(anlz.getSectionsByTag('ZZZZ').length, 2)
        t.true(anlz.removeSectionByTag('ZZZZ'))
        t.is(anlz.removeSectionsByTag('ZZZZ'), 1)
        t.is(anlz.removeSectionsByTag('ZZZZ'), 0)
        anlz.pushSection(unknown)
        anlz.dump()
        const reloaded = new Anlz(path)
        t.deepEqual(reloaded.getSectionByTag('ZZZZ'), unknown)
        t.true(anlz.removeBeatGrid())
        t.false(anlz.removeBeatGrid())
        t.true(anlz.removePath())
        anlz.removeSection(0)
        t.is(anlz.sectionCount(), 0)
    })
}

test('ANLZ rejects invalid edits without changing the handler', t => {
    const anlz = Anlz.newDat(temporaryPath(t, 'invalid.DAT'))
    anlz.setWaveformPreview({ data: [{ height: 1, whiteness: 0 }] })
    for (const action of [
        () => anlz.setWaveformPreview({ data: [{ height: 32, whiteness: 0 }] }),
        () => anlz.setWaveformPreview({ data: [{ height: 256, whiteness: 0 }] }),
        () => anlz.setWaveformPreview({ data: [{ height: 1.5, whiteness: 0 }] }),
        () => anlz.setWaveformPreview({ data: [{ height: NaN, whiteness: 0 }] }),
        () => anlz.setSection({ tag: 'PQTZ', content: { data: [] } }),
        () => anlz.setSection({ tag: 'Unknown', content: { code: [80, 81, 84, 90], headerData: [], contentData: [] } }),
        () => anlz.removeSection(99),
        () => anlz.removeSection(-4294967296),
        () => anlz.removeSection(0.5),
        () => anlz.contains('not-a-tag'),
    ]) {
        t.throws(action, { instanceOf: Error })
        t.is(anlz.sectionCount(), 1)
        t.is(anlz.getWaveformPreview().data[0].height, 1)
    }
})
