// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

import test from 'ava'
import { OneLibrary, OneLibraryPlaylistType, PlaylistType } from '../index.js'
import { cleanupFixtures, fixtureCopy } from './helpers/fixtures.mjs'

test.afterEach.always(cleanupFixtures)

test.beforeEach(t => {
    t.context.db = new OneLibrary(fixtureCopy(t, 'oneLibrary'))
})

test('database playlist enums have distinct exports', t => {
    t.not(OneLibraryPlaylistType, PlaylistType)
    const row = t.context.db.createPlaylist('enum API', null, null)
    t.is(row.attribute, OneLibraryPlaylistType.List)
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
