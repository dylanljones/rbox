// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

import test from 'ava'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { OneLibrary, PlaylistType } from '../index.js'
import { cleanupFixtures, fixtureCopy, temporaryPath } from './helpers/fixtures.mjs'

test.afterEach.always(cleanupFixtures)

test.beforeEach(t => {
    t.context.db = OneLibrary.fromPath(fixtureCopy(t, 'oneLibrary'))
})

test('directory and explicit file constructors', t => {
    const file = temporaryPath(t, 'exportLibrary.db')
    const db = new OneLibrary(path.dirname(file))
    t.true(t.context.db instanceof OneLibrary)
    t.is(db.getContents().length, t.context.db.getContents().length)
    t.throws(() => new OneLibrary(file), { instanceOf: Error })
    t.throws(() => OneLibrary.fromPath(path.dirname(file)), { instanceOf: Error })
})

test('missing database constructors do not create files', t => {
    const missing = temporaryPath(t, 'missing')
    t.throws(() => new OneLibrary(missing), { instanceOf: Error })
    t.throws(() => OneLibrary.fromPath(missing), { instanceOf: Error })
    t.false(existsSync(missing))
})

test('OneLibrary uses the shared playlist enum', t => {
    t.not(PlaylistType)
    const row = t.context.db.createPlaylist('enum API', null, null)
    t.is(row.attribute, PlaylistType.List)
})

test('raw getters and argument conversion', t => {
    const db = t.context.db
    t.true(db instanceof OneLibrary)
    const rows = db.getContents()
    t.true(Array.isArray(rows))
    const row = rows[0]
    t.is(typeof row.id, 'number')
    t.is(typeof db.getContentById(row.id), 'object')
    t.true(db.getContentById(2147483647) == null)
    t.true(Array.isArray(db.getAlbums()))
    t.true(db.getAlbumById(2147483647) == null)
    t.false('artist' in row)
    t.throws(() => db.getContentById([]), { instanceOf: Error })
})

test('content lookup accepts signed master identities', t => {
    const db = t.context.db
    const row = db.getContents()[0]
    row.masterContentId = 2000000123
    row.masterDbId = -795645379
    db.updateContent(row)
    const found = db.getContentByMasterContentId(row.masterContentId, row.masterDbId)
    t.is(found.id, row.id)
    t.is(found.masterDbId, -795645379)
    t.true(db.getContentByMasterContentId(row.masterContentId, 42) == null)
    t.true(db.getContentByMasterContentId(0, 0) == null)
    const batch = db.getContentsByMasterContentIds(
        [row.masterContentId, 2000000125, row.masterContentId, 0], row.masterDbId)
    t.deepEqual(batch.map(item => item.id), [row.id])
    t.deepEqual(db.getContentsByMasterContentIds([], row.masterDbId), [])
    t.deepEqual(db.getContentsByMasterContentIds([row.masterContentId], 42), [])
    t.throws(() => db.getContentsByMasterContentIds(['invalid'], row.masterDbId), { instanceOf: Error })
    t.throws(() => db.getContentByMasterContentId([], row.masterDbId), { instanceOf: Error })
})

test('album mutation signatures', t => {
    const db = t.context.db
    const row = db.createAlbum('API album', null, null)
    t.is(typeof row.id, 'number')
    t.is(typeof db.getAlbumByName('API album'), 'object')
    row.name = 'API renamed album'
    t.is(typeof db.updateAlbum(row), 'object')
    t.is(typeof db.deleteAlbum(row.id), 'number')
})

test('content views expose converted rows and optional fields', t => {
    const db = t.context.db
    const row = db.getContents()[0]
    row.artistId = db.createArtist('API artist').id
    t.is(typeof db.updateContent(row), 'object')
    const view = db.getContentWithRelations(row.id)
    t.is(typeof view.content.id, 'number')
    t.is(typeof view.artist.name, 'string')
    for (const field of ['remixer', 'originalArtist', 'composer', 'lyricist', 'album', 'genre', 'label', 'key', 'color', 'image']) {
        // Optional object properties may be omitted by N-API.
        t.true(view[field] == null || typeof view[field].id === 'number')
    }
    const views = db.getContentsWithRelations()
    t.true(Array.isArray(views))
    t.is(typeof views[0].content, 'object')
    t.true(db.getContentWithRelations(2147483647) == null)
})

test('album views expose converted rows and optional fields', t => {
    const db = t.context.db
    const artist = db.createArtist('API album artist')
    const image = db.createImage('api-artwork.jpg')
    const row = db.createAlbum('API album', artist.id, image.id)
    const view = db.getAlbumWithRelations(row.id)
    t.is(typeof view.album.name, 'string')
    t.is(typeof view.artist.name, 'string')
    t.is(typeof view.image.id, 'number')
    const views = db.getAlbumsWithRelations()
    t.true(Array.isArray(views))
    t.is(typeof views[0].album, 'object')
    const absent = db.createAlbum('API optional fields', null, null)
    t.true(db.getAlbumWithRelations(absent.id).artist == null)
    t.true(db.getAlbumWithRelations(absent.id).image == null)
    t.true(db.getAlbumWithRelations(2147483647) == null)
})

test('category views expose nested rows', t => {
    const db = t.context.db
    const item = db.createMenuItem('API menu', 0)
    const row = db.createCategory(item.id, null, null)
    const view = db.getCategoryWithRelations(row.id)
    t.is(typeof view.category.id, 'number')
    t.is(typeof view.menuItem.name, 'string')
    const views = db.getCategoriesWithRelations()
    t.true(Array.isArray(views))
    t.is(typeof views[0].category, 'object')
    t.true(db.getCategoryWithRelations(2147483647) == null)
})

test('shared tracks expose numeric identities and album artists', t => {
    const db = t.context.db
    const row = db.getContents()[0]
    const artist = db.createArtist('Shared track album artist')
    const album = db.createAlbum('Shared track album', artist.id, null)
    row.albumId = album.id
    row.masterDbId = -795645379
    db.updateContent(row)
    t.is(db.getContentWithRelations(row.id).albumArtist.id, artist.id)
    const track = db.getTrackById(row.id)
    t.is(track.id, row.id)
    t.is(track.masterDbId, 3499321917)
    t.is(track.masterContentId, row.masterContentId)
    t.is(track.albumArtist, artist.name)
    t.is(db.getTrackByPath(row.path.replaceAll('/', '\\')).id, row.id)
    t.true(db.getTracks().some(item => item.id === row.id))
    t.deepEqual(db.getTracksByIds([row.id, row.id, 2147483647]).map(item => item.id), [row.id])
    t.deepEqual(db.getTracksByIds([]), [])
    t.true(db.getTrackById(2147483647) == null)
    t.true(db.getTrackByPath('/missing-track.wav') == null)
    t.throws(() => db.getTrackById('not an integer'))
    row.rating = 6
    db.updateContent(row)
    t.throws(() => db.getTrackById(row.id), {instanceOf: Error})
})
