// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

// Master database API contracts and conversion of raw and resolved models.

import test from 'ava'
import { MasterDb } from '../index.js'
import { cleanupFixtures, fixtureCopy } from './helpers/fixtures.mjs'

test.afterEach.always(cleanupFixtures)

test.beforeEach(t => {
    t.context.db = new MasterDb(fixtureCopy(t, 'masterdb'))
})

test('raw getters and argument conversion', t => {
    const db = t.context.db
    t.true(db instanceof MasterDb)
    t.is(typeof db.addContent, 'function')
    const rows = db.getContents()
    t.true(Array.isArray(rows))
    const row = rows[0]
    t.is(typeof row.id, 'string')
    t.is(typeof db.getContentById(row.id), 'object')
    t.true(db.getContentById('missing-id') == null)
    t.true(Array.isArray(db.getAlbums()))
    t.true(db.getAlbumById('missing-id') == null)
    t.false('artist' in row)
    t.throws(() => db.getContentById([]), { instanceOf: Error })
})

test('resolved track helpers expose core models and translate errors', t => {
    const db = t.context.db
    const row = db.getContents()[0]
    row.artistId = db.createArtist('Track helper artist').id
    row.bpm = 12825
    row.imagePath = '/art/cover.jpg'
    row.analysisDataPath = '/ANLZ/ANLZ0000.DAT'
    row.colorId = '3'
    row.djPlayCount = 123
    row.hotCueAutoLoad = 'unknown-setting'
    row.deliveryControl = 'on'
    row.deliveryComment = 'Comment'
    delete row.composerId
    db.updateContent(row)
    const track = db.getTrackById(row.id)
    t.is(track.id, Number(row.id))
    t.is(track.masterContentId, Number(row.id))
    t.is(track.masterDbId, Number(db.getProperties()[0].dbId))
    t.is(track.artworkPath, row.imagePath)
    t.is(track.analysisDataPath, row.analysisDataPath)
    t.is(track.color, 3)
    t.is(track.playCount, 123)
    t.is(track.hotCueAutoLoad, 'unknown-setting')
    t.is(track.deliveryControl, 'on')
    t.is(track.deliveryComment, 'Comment')
    t.is(track.artist, 'Track helper artist')
    t.is(track.bpm, 128.25)
    t.true(track.composer == null)
    t.is(typeof track.path, 'string')
    t.true(track.fileSize == null || typeof track.fileSize === 'bigint')
    t.is(db.getTrackByPath(track.path).id, Number(row.id))
    t.true(db.getTracks().every(item => Number.isInteger(item.id)))
    t.deepEqual(db.getTracksByIds([row.id, 'missing', row.id]).map(item => item.id), [Number(row.id)])
    t.deepEqual(db.getTracksByIds([]), [])
    t.true(db.getTrackById('missing') == null)
    t.true(db.getTrackByPath('/missing/track.mp3') == null)
    t.throws(() => db.getTracksByIds([123]))
    row.rating = 6
    db.updateContent(row)
    t.throws(() => db.getTrackById(row.id), { message: /rating/ })
})

test('album mutation signatures', t => {
    const db = t.context.db
    const row = db.createAlbum('API album', null, null, null)
    t.is(typeof row.id, 'string')
    t.is(typeof db.getAlbumByName('API album'), 'object')
    row.name = 'API renamed album'
    t.is(typeof db.updateAlbum(row), 'object')
    t.is(db.deleteAlbum(row.id), undefined)
})

test('content views expose converted rows and optional fields', t => {
    const db = t.context.db
    const row = db.getContents()[0]
    row.artistId = db.createArtist('API artist').id
    const albumArtist = db.createArtist('API album artist')
    row.albumId = db.createAlbum('API content album', albumArtist.id, null, null).id
    t.is(typeof db.updateContent(row), 'number')
    const view = db.getContentWithRelations(row.id)
    t.is(typeof view.content.id, 'string')
    t.is(typeof view.artist.name, 'string')
    t.is(view.albumArtist.id, albumArtist.id)
    for (const field of ['remixer', 'originalArtist', 'composer', 'album', 'genre', 'label', 'key', 'color']) {
        // Optional object properties may be omitted by N-API.
        t.true(view[field] == null || typeof view[field].id === 'string')
    }
    const views = db.getContentsWithRelations()
    t.true(Array.isArray(views))
    t.is(typeof views[0].content, 'object')
    t.true(db.getContentWithRelations('missing-id') == null)
})

test('album views expose converted rows and optional fields', t => {
    const db = t.context.db
    const artist = db.createArtist('API album artist')

    const row = db.createAlbum('API album', artist.id, null, null)
    const view = db.getAlbumWithRelations(row.id)
    t.is(typeof view.album.name, 'string')
    t.is(typeof view.artist.name, 'string')

    const views = db.getAlbumsWithRelations()
    t.true(Array.isArray(views))
    t.is(typeof views[0].album, 'object')
    const absent = db.createAlbum('API optional fields', null, null, null)
    t.true(db.getAlbumWithRelations(absent.id).artist == null)

    t.true(db.getAlbumWithRelations('missing-id') == null)
})
