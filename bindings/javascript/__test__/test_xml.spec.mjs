import test from 'ava'
import fs from 'fs'
import { RekordboxXml } from '../index.js'

const DEMO_XML_SRC = "../../.testdata/RBv6/demo/database.xml"
const DEMO_XML_OUT = "../../.testdata/RBv6/demo/database-out.xml"
const DEMO_XML  = "../../.testdata/RBv6/demo/database-copy.xml"

function prepareTestXml() {
    fs.writeFileSync(DEMO_XML, fs.readFileSync(DEMO_XML_SRC));
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

test("Get tracks", async (t) => {
    prepareTestXml();
    const xml = RekordboxXml.load(DEMO_XML);
    const tracks = xml.getTracks();
    console.log(`Found ${tracks.length} tracks`);
    t.is(tracks.length, 6);
});
