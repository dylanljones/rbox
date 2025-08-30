import test from 'ava'
import fs from 'fs'
import { RekordboxXml, PlaylistKeyType, PlaylistNode, PlaylistNodeType  } from '../index.js'

const DEMO_XML_SRC = "../../.testdata/RBv6/demo/database.xml"
const DEMO_XML_OUT = "../../.testdata/RBv6/demo/database-out.xml"
const DEMO_XML  = "../../.testdata/RBv6/demo/database-copy.xml"

function prepareTestXml() {
    fs.writeFileSync(DEMO_XML, fs.readFileSync(DEMO_XML_SRC));
    return DEMO_XML
}

test("Open Xml", async (t) => {
    prepareTestXml();
    const xml = RekordboxXml.load(DEMO_XML);
    t.is(true, true);
});

test("Write Xml", async (t) => {
    prepareTestXml();
    const xml = RekordboxXml.load(DEMO_XML);
    xml.dump();
    xml.dumpCopy(DEMO_XML_OUT);
    t.is(true, true);
});

// ---- Tracks ------------------------------------------------------------------------------------

test("get tracks returns expected length", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)
    const tracks = xml.getTracks()
    t.is(tracks.length, 6)
})

test("get track by index returns some and none", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const track1 = xml.getTrack(0)
    t.truthy(track1)

    const trackNone = xml.getTrack(6)
    t.is(trackNone, null)
})

test("get track by id returns some and none", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const track = xml.getTrackById("253529738")
    t.truthy(track)

    const trackNone = xml.getTrackById("12345")
    t.is(trackNone, null)
})

test("track location decode", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const track = xml.getTrackById("253529738")
    t.truthy(track)
    const expected = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3"
    t.is(track.location, expected)
})

test("track location encode persists after dumpCopy", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const trackId = "253529738"
    const track = xml.getTrackById(trackId)
    t.truthy(track)

    const newLocation = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 2.mp3"
    track.location = newLocation
    xml.updateTrack(track);

    xml.dumpCopy(DEMO_XML_OUT)
    const xml2 = RekordboxXml.load(DEMO_XML_OUT)
    const track2 = xml2.getTrackById(trackId)
    t.truthy(track2)
    t.is(track2.location, newLocation)
})

test("get track by location returns some and none", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3"
    t.truthy(xml.getTrackByLocation(loc))

    const locNone = "C:/Music/PioneerDJ/Demo Tracks/Demo Track.mp3"
    t.is(xml.getTrackByLocation(locNone), null)
})


test("get track by key supports Location and TrackID", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const id = "253529738"
    const loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3"

    t.truthy(xml.getTrackByKey(loc, PlaylistKeyType.Location))
    t.truthy(xml.getTrackByKey(id, PlaylistKeyType.TrackID))
})


test("add track increases track count and is retrievable", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const id = "1234567"
    const loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track.mp3"

    t.is(xml.getTracks().length, 6)
    const newTrack = {
        trackid: id,
        location: loc,
    };
    xml.addTrack(newTrack)
    t.is(xml.getTracks().length, 7)
    t.truthy(xml.getTrackById(id))
})


test("new track helper increases track count and is retrievable", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const id = "1234567"
    const loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track.mp3"

    t.is(xml.getTracks().length, 6)
    xml.newTrack(id, loc)
    t.is(xml.getTracks().length, 7)
    t.truthy(xml.getTrackById(id))
})


test("remove track decreases track count", (t) => {
    const path = prepareTestXml()
    const xml = RekordboxXml.load(path)

    const id = "253529738"
    t.is(xml.getTracks().length, 6)
    xml.removeTrack(id)
    t.is(xml.getTracks().length, 5)
})

// ---- Playlists ---------------------------------------------------------------------------------


test("new playlist folder node", (t) => {
    const folder = PlaylistNode.folder("Test Folder")
    t.is(folder.name, "Test Folder")
    t.is(folder.nodeType, PlaylistNodeType.Folder)
})

test("new playlist node", (t) => {
    const playlist = PlaylistNode.playlist("Test Playlist", PlaylistKeyType.TrackID)
    t.is(playlist.name, "Test Playlist")
    t.is(playlist.nodeType, PlaylistNodeType.Playlist)
    t.is(playlist.keyType, PlaylistKeyType.TrackID)
})

test("add node to folder", (t) => {
    const folder = PlaylistNode.folder("Parent Folder")
    const child = PlaylistNode.folder("Child Folder")
    folder.addNode(child)
    t.is(folder.nodes.length, 1)
    t.is(folder.nodes[0].name, "Child Folder")
})

test("add track to playlist by key", (t) => {
    const playlist = PlaylistNode.playlist("Test Playlist", PlaylistKeyType.TrackID)
    playlist.addTrack({key: "123"})
    t.is(playlist.tracks.length, 1)
    t.is(playlist.tracks[0].key, "123")
})


test("remove node from folder by index", (t) => {
    const folder = PlaylistNode.folder("Parent Folder")
    folder.addNode(PlaylistNode.folder("Child Folder"))
    folder.removeNode(0)
    t.is(folder.nodes.length, 0)
})


test("remove track from playlist by key", (t) => {
    const playlist = PlaylistNode.playlist("Test Playlist", PlaylistKeyType.TrackID)
    playlist.addTrack({key: "123"})
    playlist.removeTrack(0)
    t.is(playlist.tracks.length, 0)
})
