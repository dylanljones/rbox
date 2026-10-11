// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

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
    t.is(typeof db.updateContent(row), 'number')
    const view = db.getContentWithRelations(row.id)
    t.is(typeof view.content.id, 'string')
    t.is(typeof view.artist.name, 'string')
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
