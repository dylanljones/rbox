// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

import test from 'ava'
import { writeFileSync } from 'node:fs'
import { RekordboxXml, PlaylistKeyType, PlaylistNode, PlaylistNodeType } from '../index.js'
import { cleanupFixtures, fixtureCopy, temporaryPath } from './helpers/fixtures.mjs'

test.afterEach.always(cleanupFixtures)

test('XML load failures throw JavaScript errors', t => {
    t.throws(() => RekordboxXml.load(temporaryPath(t, 'missing.xml')), { instanceOf: Error })
    const path = temporaryPath(t, 'malformed.xml')
    writeFileSync(path, '<invalid>')
    t.throws(() => RekordboxXml.load(path), { instanceOf: Error })
})

test('XML constructors, getters and optional results', t => {
    const xml = RekordboxXml.load(fixtureCopy(t, 'xml'))
    t.true(xml instanceof RekordboxXml)
    const tracks = xml.getTracks()
    t.true(Array.isArray(tracks))
    const track = tracks[0]
    t.is(typeof track.trackid, 'string')
    t.is(typeof xml.getTrack(0), 'object')
    t.is(typeof xml.getTrackById(track.trackid), 'object')
    t.is(typeof xml.getTrackByLocation(track.location), 'object')
    t.is(typeof xml.getTrackByKey(track.trackid, PlaylistKeyType.TrackID), 'object')
    t.is(xml.getTrackById('missing-id'), null)
    t.throws(() => xml.getTrack({}), { instanceOf: Error })
})

test('XML mutation and writer signatures', t => {
    const xml = RekordboxXml.load(fixtureCopy(t, 'xml'))
    const track = xml.newTrack('api-track', '/api/track.mp3')
    t.is(typeof track, 'object')
    track.name = 'API title'
    t.is(xml.updateTrack(track), undefined)
    t.is(xml.removeTrack('api-track'), undefined)
    t.is(xml.addTrack({ trackid: 'api-added-track', location: '/api/added.mp3' }), undefined)
    t.is(xml.dump(), undefined)
    t.is(xml.dumpCopy(temporaryPath(t, 'copy.xml')), undefined)
})

test('playlist factories, enums and collection arguments', t => {
    const folder = PlaylistNode.folder('API folder')
    const playlist = PlaylistNode.playlist('API playlist', PlaylistKeyType.TrackID)
    t.true(folder instanceof PlaylistNode)
    t.true(playlist instanceof PlaylistNode)
    t.is(folder.nodeType, PlaylistNodeType.Folder)
    t.is(playlist.keyType, PlaylistKeyType.TrackID)
    t.is(folder.addNode(playlist), undefined)
    t.true(Array.isArray(folder.nodes))
    t.is(playlist.addTrack({ key: 'api-track' }), undefined)
    t.true(Array.isArray(playlist.tracks))
    t.is(playlist.removeTrack(0), undefined)
    t.is(folder.removeNode(0), undefined)
})
