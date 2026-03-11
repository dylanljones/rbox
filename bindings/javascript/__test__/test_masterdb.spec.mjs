import test from 'ava'
import fs from 'fs'
import { MasterDb } from '../index.js'

const DEMO_DB_SRC = "../../.testdata/RBv6/demo/rekordbox/master.db"
const DEMO_DB  = "../../.testdata/RBv6/demo/rekordbox/master-copy.db"

function prepareTestDb() {
    fs.writeFileSync(DEMO_DB, fs.readFileSync(DEMO_DB_SRC));
}

test("Open database", async (t) => {
    prepareTestDb();
    const db = new MasterDb(DEMO_DB);
    t.is(true, true);
});

test("Get USN", async (t) => {
    prepareTestDb();
    const db = new MasterDb(DEMO_DB);
    const usn = db.getLocalUsn();
    t.is(usn > 0, true);
})

test("Get contents", async (t) => {
    prepareTestDb();
    const db = new MasterDb(DEMO_DB);
    const tracks = db.getContents();
    t.is(tracks.length > 0, true);
});
