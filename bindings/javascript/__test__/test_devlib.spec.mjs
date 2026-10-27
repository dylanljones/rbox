// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

// Device Library lifecycle, model conversions, and explicit persistence contracts.

import test from 'ava'
import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'
import {
    DeviceLibrary, DevAlbum, DevRecommendedLike,
    PlaylistType,
} from '../index.js'
import { cleanupFixtures, fixtureCopy, temporaryPath } from './helpers/fixtures.mjs'

test.afterEach.always(cleanupFixtures)

function createLibrary(t) {
    const directory = temporaryPath(t, 'library')
    const db = DeviceLibrary.create(directory, {
        date: '2026-10-27', deviceName: 'Binding test', myTagMasterDbId: 2 ** 32 - 1,
    })
    t.context.db = db
    return { db, directory }
}

test('demo directory and explicit paths expose both database files', t => {
    const standard = fixtureCopy(t, 'deviceLibrary')
    const extended = fixtureCopy(t, 'deviceLibraryExtended')
    const db = new DeviceLibrary(path.dirname(standard))
    t.context.db = db
    t.true(db.hasExtended())
    const explicit = DeviceLibrary.fromPaths(standard, extended)
    t.is(explicit.getContents().length, db.getContents().length)
    const content = db.getContents()[0]
    t.is(Object.getPrototypeOf(content), Object.prototype)
    t.deepEqual({ ...content }, content)
    t.deepEqual(JSON.parse(JSON.stringify(content)), content)
    t.false('raw' in content)
    t.false('isrcOverride' in content)
    t.true(Array.isArray(db.getAlbums()))
    t.true(db.getContentById(2 ** 32 - 1) == null)
    t.true(Array.isArray(db.getMyTags()))
    t.true(Array.isArray(db.getDatabaseProperties()))
    t.throws(() => DeviceLibrary.fromPaths(standard, standard), { instanceOf: Error })
})

test('standard-only libraries retain an absent extended database', t => {
    const standard = fixtureCopy(t, 'deviceLibrary')
    const db = DeviceLibrary.fromPaths(standard)
    t.context.db = db
    t.false(db.hasExtended())
    t.throws(() => db.getMyTags(), { instanceOf: Error })
    t.true(db.save().extended == null)
    const copy = temporaryPath(t, 'copy')
    db.saveAs(copy)
    t.true(existsSync(path.join(copy, 'export.pdb')))
    t.false(existsSync(path.join(copy, 'exportExt.pdb')))
    t.throws(() => DeviceLibrary.fromPaths(standard, path.join(copy, 'exportExt.pdb')),
        { instanceOf: Error })
})

test('demo content patches preserve untouched metadata and leave no-op saves unchanged', t => {
    const standard = fixtureCopy(t, 'deviceLibrary')
    const db = DeviceLibrary.fromPaths(standard)
    t.context.db = db
    const before = readFileSync(standard)
    const original = db.getContents()[0]
    t.deepEqual(db.updateContent(original.id, {}), original)
    t.is(db.save().standard.bytesWritten, 0n)
    t.deepEqual(readFileSync(standard), before)
    const expected = { ...original, comment: 'Updated through plain-object patch' }
    t.deepEqual(db.updateContent(original.id, { comment: expected.comment }), expected)
    db.save()
    const reopened = DeviceLibrary.fromPaths(standard)
    t.deepEqual(reopened.getContentById(original.id), expected)
})

test('missing paths and creation errors do not overwrite exports', t => {
    const missing = temporaryPath(t, 'missing')
    t.throws(() => new DeviceLibrary(missing), { instanceOf: Error })
    t.false(existsSync(missing))
    const { directory } = createLibrary(t)
    const standard = path.join(directory, 'export.pdb')
    const before = readFileSync(standard)
    t.throws(() => DeviceLibrary.create(directory, {
        date: '', deviceName: 'replacement', myTagMasterDbId: 1,
    }), { instanceOf: Error })
    t.deepEqual(readFileSync(standard), before)
    t.throws(() => DeviceLibrary.create(missing, {
        date: '', deviceName: '', myTagMasterDbId: -1,
    }), { instanceOf: Error })
    t.false(existsSync(missing))
})

test('album inputs, borrowed snapshots, and explicit relations round trip', t => {
    const { db } = createLibrary(t)
    const artist = db.insertArtist({ name: 'Artist' })
    const album = db.insertAlbum({
        name: 'Album', artistId: artist.id, unknown04: 0, unknown10: 0,
    })
    t.true(album instanceof DevAlbum)
    album.name = 'Renamed'
    t.is(db.getAlbumById(album.id).name, 'Album')
    const updated = db.updateAlbum(album)
    t.is(album.name, 'Renamed') // Passing a snapshot does not consume it.
    t.is(updated.name, 'Renamed')
    t.false('raw' in updated)
    t.is(db.getAlbumWithRelations(album.id).artist.name, 'Artist')
    t.is(db.getAlbumsWithRelations()[0].album.id, album.id)
    t.is(db.getAlbumByName('Renamed').id, album.id)
    t.throws(() => db.insertAlbum({ name: 'Invalid' }), { instanceOf: Error })
    t.throws(() => db.updateAlbum({ id: album.id }), { instanceOf: Error })
    t.throws(() => db.getAlbumById(-1), { instanceOf: Error })
    t.is(db.deleteAlbum(album.id), 1)
    t.true(db.getAlbumById(album.id) == null)
})

test('plain content inputs and results retain shared tracks and master identities', t => {
    const { db } = createLibrary(t)
    const input = {
        path: '/Contents/track.wav', title: 'Track',
        masterContentId: 2 ** 32 - 1, masterDbId: 2 ** 32 - 1, sampleRate: 2 ** 32 - 1,
    }
    const content = db.insertContent(input)
    t.is(Object.getPrototypeOf(content), Object.prototype)
    t.deepEqual(JSON.parse(JSON.stringify(content)), content)
    t.is(content.fileName, 'track.wav')
    t.is(content.comment, '')
    t.is(content.rating, 0)
    t.false('creation' in content)
    t.false('raw' in content)
    t.is(input.title, 'Track')
    t.is(content.sampleRate, input.sampleRate)
    t.is(db.getContentByMasterContentId(input.masterContentId, input.masterDbId).id, content.id)
    t.deepEqual(db.getContentsByMasterContentIds([input.masterContentId], input.masterDbId)
        .map(row => row.id), [content.id])
    const track = db.getTrackById(content.id)
    t.is(track.title, 'Track')
    t.is(typeof track.fileSize, 'bigint')
    t.is(db.getTrackByPath(input.path).title, 'Track')
    t.is(db.getTracksByIds([content.id, content.id, 12345]).length, 1)
    t.is(db.getTracks().length, 1)
    t.true(db.getContentWithRelations(content.id).artist == null)
    t.is(Object.getPrototypeOf(db.getContentWithRelations(content.id).content), Object.prototype)
    t.is(db.getContentsWithRelations().length, 1)
    content.title = 'Edited'
    t.is(db.getContentById(content.id).title, 'Track')
    t.is(db.updateContent(content.id, { title: content.title }).title, 'Edited')
})

test('content insertion requires a path and defaults omitted metadata', t => {
    const { db } = createLibrary(t)
    for (const input of [{}, { path: 123 }, { path: '' }, { path: '/track.wav', title: 123 },
        { path: '/track.wav', comment: null }]) {
        t.throws(() => db.insertContent(input), { instanceOf: Error })
    }
    t.is(db.getContents().length, 0)
    const content = db.insertContent({ path: '/Contents/default.wav' })
    t.is(content.fileName, 'default.wav')
    t.is(content.title, '')
    t.is(content.sampleRate, 0)
    const override = db.insertContent({
        path: '/Contents/custom.wav', fileName: 'override.wav', title: undefined,
    })
    t.is(override.fileName, 'override.wav')
    t.is(override.title, '')
    t.is(override.comment, '')
})

test('content patches preserve omitted values, validate changes, and survive saving', t => {
    const { db, directory } = createLibrary(t)
    const artist = db.createArtist('Artist')
    const content = db.insertContent({ path: '/track.wav', title: 'Original', rating: 5 })
    db.updateContent(content.id, { artistId: artist.id, comment: 'Keep this' })
    const changes = Object.freeze({ title: 'Updated', masterDbId: 2 ** 32 - 1 })
    const updated = db.updateContent(content.id, changes)
    t.is(updated.title, 'Updated')
    t.is(updated.comment, 'Keep this')
    t.is(updated.artistId, artist.id)
    t.is(updated.rating, 5)
    t.is(content.title, 'Original')
    t.deepEqual(db.updateContent(content.id, {}), updated)
    t.deepEqual(db.updateContent(content.id, { title: undefined, rating: undefined }), updated)
    // Failed conversions or binary range checks must not partially apply a patch.
    for (const patch of [{ title: 123 }, { title: 'Invalid', rating: 256 }, { artistId: -1 },
        { rating: null }]) {
        t.throws(() => db.updateContent(content.id, patch), { instanceOf: Error })
        t.deepEqual(db.getContentById(content.id), updated)
    }
    t.throws(() => db.updateContent(99999, {}), { message: /no model with ID 99999/ })
    t.throws(() => db.updateContent(-1, {}), { instanceOf: Error })
    const cleared = db.updateContent(content.id, { title: '', rating: 0 })
    t.is(cleared.title, '')
    t.is(cleared.rating, 0)
    db.save()
    const reopened = new DeviceLibrary(directory)
    t.deepEqual(reopened.getContentById(content.id), cleared)
    t.is(reopened.getContentWithRelations(content.id).artist.name, 'Artist')
})

test('catalog and history operations preserve native snapshots', t => {
    const { db } = createLibrary(t)
    const genre = db.createGenre('House')
    const label = db.createLabel('Label')
    const artwork = db.createArtwork('/artwork.jpg')
    const key = db.createKey('Am', 1)
    t.is(db.getGenreByName('House').id, genre.id)
    t.is(db.getLabelByName('Label').id, label.id)
    t.is(db.getArtworkByPath('/artwork.jpg').id, artwork.id)
    t.is(db.getKeyByName('Am').secondaryId, key.secondaryId)
    const content = db.createContent('/track.wav')
    const history = db.createHistory('Played')
    const entry = db.insertHistoryContent({ historyId: history.id, contentId: content.id, seq: 1 })
    t.is(db.getHistoryEntries(history.id)[0].seq, 1)
    t.is(db.getHistoryContents(history.id)[0].id, content.id)
    t.is(db.deleteHistoryContent(entry), 1)
    t.is(entry.contentId, content.id)
    t.is(db.getHistoryEntries(history.id).length, 0)
})

test('playlist paths, trees, optional moves, and pair arrays are converted', t => {
    const { db } = createLibrary(t)
    const folder = db.createPlaylistFolder('Sets')
    const playlist = db.createPlaylist('Warmup', folder.id)
    t.is(playlist.attribute, PlaylistType.List)
    t.is(db.getPlaylistByPath(['Sets', 'Warmup']).id, playlist.id)
    t.is(db.getPlaylistTree()[0].children[0].playlist.id, playlist.id)
    t.is(db.getPlaylistAttribute(playlist.id), PlaylistType.List)
    t.true(db.isPlaylist(playlist.id))
    const first = db.createContent('/first.wav')
    const second = db.createContent('/second.wav')
    db.createPlaylistContent(playlist.id, first.id)
    db.insertPlaylistContent({ playlistId: playlist.id, contentId: second.id })
    db.movePlaylistContents([[playlist.id, second.id]], 1)
    t.deepEqual(db.getPlaylistContents(playlist.id).map(row => row.id), [second.id, first.id])
    t.true(db.validatePlaylistContentSeqNumbers(playlist.id))
    db.movePlaylist(playlist.id, null, 0)
    t.is(db.getPlaylistById(playlist.id).parentId, folder.id)
    db.movePlaylists([playlist.id], 0)
    t.is(db.getPlaylistById(playlist.id).parentId, 0)
    t.is(db.deletePlaylistContents([[playlist.id, first.id]]), 1)
    t.throws(() => db.getPlaylistByPath([123]), { instanceOf: Error })
})

test('My Tag inputs validate fixed arrays and support extended-file relationships', t => {
    const { db } = createLibrary(t)
    const content = db.createContent('/tagged.wav')
    const tag = db.createMyTag(2 ** 32 - 1, 'Tag')
    const input = {
        contentId: content.id, myTagId: tag.id, unknown00: 0,
        reservedStringMarker: 3, reservedTail: [0, 128, 255],
    }
    const entry = db.insertMyTagContent(input)
    t.is(db.getMyTagContents(tag.id)[0].id, content.id)
    t.is(db.getMyTagByContentId(content.id)[0].id, tag.id)
    t.throws(() => db.insertMyTagContent({ ...input, reservedTail: [0, 1] }), { instanceOf: Error })
    t.throws(() => db.insertMyTagContent({ ...input, reservedTail: [0, 1, 2, 3] }), { instanceOf: Error })
    t.is(db.getMyTagAssociations().length, 1)
    t.is(db.deleteMyTagContentEntry(entry), 1)
    t.is(db.getMyTagEntries(tag.id).length, 0)
})

test('recommended likes preserve bigint values and reject overflow', t => {
    const { db } = createLibrary(t)
    const maximum = (1n << 64n) - 1n
    const row = db.insertRecommendLike({
        contentId1: 1, contentId2: 2, unknown00: maximum, unknown10: 0, unknown14: 0,
    })
    t.true(row instanceof DevRecommendedLike)
    t.is(row.unknown00, maximum)
    row.unknown00 = 0n
    t.is(row.unknown00, 0n)
    t.throws(() => { row.unknown00 = -1n }, { instanceOf: Error })
    t.throws(() => { row.unknown00 = maximum + 1n }, { instanceOf: Error })
    t.is(row.unknown00, 0n)
    t.is(db.getRecommendLikeByContentId(1)[0].unknown00, maximum)
    t.throws(() => db.createRecommendLike(3, 4, -1n, 0, 0), { instanceOf: Error })
    t.throws(() => db.createRecommendLike(3, 4, maximum + 1n, 0, 0), { instanceOf: Error })
    t.is(db.createRecommendLike(3, 4, maximum, 0, 0).unknown00, maximum)
    t.is(db.deleteRecommendLike(row), 1)
    t.is(db.getRecommendLikes().length, 1)
})

test('saves report bigint statistics and saveAs retains the source', t => {
    const { db, directory } = createLibrary(t)
    const standard = path.join(directory, 'export.pdb')
    const extended = path.join(directory, 'exportExt.pdb')
    const before = readFileSync(standard)
    const beforeExtended = readFileSync(extended)
    t.is(db.save().standard.bytesWritten, 0n)
    const artist = db.createArtist('Pending')
    db.createMyTag(10, 'Pending tag')
    t.deepEqual(readFileSync(standard), before)
    t.deepEqual(readFileSync(extended), beforeExtended)
    const copy = temporaryPath(t, 'copy')
    db.saveAs(copy)
    t.deepEqual(readFileSync(standard), before)
    t.is(new DeviceLibrary(copy).getArtistById(artist.id).name, 'Pending')
    t.throws(() => db.saveAs(copy), { instanceOf: Error })
    const stats = db.save()
    t.true(stats.standard.bytesWritten > 0n)
    t.true(stats.extended.bytesWritten > 0n)
    t.is(db.save().standard.pagesWritten, 0n)
    const reopened = new DeviceLibrary(directory)
    t.is(reopened.getArtistById(artist.id).name, 'Pending')
    t.is(reopened.getMyTagById(10).name, 'Pending tag')
    artist.name = 'Page save'
    db.updateArtist(artist)
    t.true(db.savePages().standard.pagesWritten > 0n)
    t.is(new DeviceLibrary(directory).getArtistById(artist.id).name, 'Page save')
    t.is(db.savePages().standard.bytesWritten, 0n)
})

test('browse and property snapshots can be read and explicitly updated', t => {
    const { db } = createLibrary(t)
    t.is(db.getFirstDatabaseProperty().myTagMasterDbId, 2 ** 32 - 1)
    const property = db.getFirstProperty()
    property.name = 'Renamed device'
    t.is(db.updateProperty(property).name, 'Renamed device')
    t.is(db.getPropertyByDevice('Renamed device').name, 'Renamed device')
    const columns = db.getBrowseColumns()
    t.true(columns.length > 0)
    t.is(db.getBrowseColumnById(columns[0].id).name, columns[0].name)
    const categories = db.getBrowseCategories()
    t.true(categories.length > 0)
    t.is(db.updateBrowseCategory(categories[0]).columnId, categories[0].columnId)
    t.true(db.getBrowseSorts().length > 0)
    t.true(db.getColors().length > 0)
})
