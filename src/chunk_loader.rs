use inflate::inflate_bytes_zlib;
use deflate::deflate_bytes_zlib;
use fnv::FnvHashMap;
use rusqlite::{ Connection, Row, DatabaseName, Error as SqliteError };

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use chunk::Chunk;
use chunk_manager::{ ChunkState, ChunkStates };
use math::*;
use utils::SETTINGS_MUT;

pub enum Request {
    Load(ChunkCoord),
    Save(ChunkCoord, Box<Chunk>),
    Close,
}

type Response = (ChunkCoord, Box<Chunk>);

const DATABASE_INITIALIZER: &str = r"
    CREATE TABLE chunks (
        x           INTEGER NOT NULL,
        z           INTEGER NOT NULL,
        block_data  BLOB NOT NULL,
        PRIMARY KEY(x, z)
    );
";



pub struct ChunkLoader {
    // An option because we need to be able to move the thread handle when
    // calling .join() in drop.
    thread_handle: Option<thread::JoinHandle<()>>,

    rx_resp: mpsc::Receiver<Response>,
    tx_req: mpsc::Sender<Request>,
}

impl ChunkLoader {
    pub fn new(path: PathBuf) -> (ChunkLoader, ChunkStates) {
        let (tx_req, rx_req) = mpsc::channel();
        let (tx_resp, rx_resp) = mpsc::channel();

        let conn = Connection::open(&path).unwrap();
        init_database(&conn);
        let chunk_states = get_chunk_states(&conn);
        let thread_handle = thread::spawn(move || {
            database_handler(conn, rx_req, tx_resp);
        });

        let chunk_loader = ChunkLoader {
            tx_req,
            rx_resp,
            thread_handle: Some(thread_handle),
        };

        (chunk_loader, chunk_states)
    }

    pub fn enqueue_unload(&mut self, coord: ChunkCoord, chunk: Box<Chunk>) {
        self.tx_req.send(Request::Save(coord, chunk)).unwrap();
    }

    pub fn enqueue_load(&mut self, coord: ChunkCoord) {
        self.tx_req.send(Request::Load(coord)).unwrap();
    }

    pub fn iter_loaded(&mut self) -> ResponseIter {
        ResponseIter(self.rx_resp.try_iter())
    }
}

impl Drop for ChunkLoader {
    fn drop(&mut self) {
        self.tx_req.send(Request::Close).unwrap();
        self.thread_handle.take().unwrap().join();
    }
}

fn init_database(conn: &Connection) {
    let num_tables: i64 = conn.query_row("SELECT count(*) FROM SQLITE_MASTER", &[], |row| row.get(0)).unwrap();
    if num_tables == 0 {
        println!("Creating chunks table");
        conn.execute_batch(DATABASE_INITIALIZER).unwrap();
    } else {
        println!("Database had {} tables", num_tables);
    }
}

fn get_chunk_states(conn: &Connection) -> ChunkStates {
    let mut stmt = conn.prepare("SELECT x, z FROM chunks").unwrap();
    let mut result = ChunkStates::new();
    let mut iter = stmt.query(&[]).unwrap();
    while let Some(Ok(row)) = iter.next() {
        let x = row.get(0);
        let z = row.get(1);
        result.set(ChunkCoord::new(x, z), ChunkState::Saved);
    }
    result
}

fn database_handler(mut conn: Connection, rx: mpsc::Receiver<Request>, tx: mpsc::Sender<Response>) {
    // conn.blob_open(DatabaseName::Main, "chunks", "block_data", 0, false);

    let mut requests = Vec::new();
    let mut done = false;
    while !done {
        for request in rx.try_iter() {
            if let Request::Close = request {
                done = true;
                break;
            } else {
                requests.push(request);
            }
        }

        let trans = conn.transaction().unwrap();
        for req in requests.drain(..) {
            match req {
                Request::Load(coord) => {
                    let mut load_stmt = trans.prepare_cached("SELECT block_data FROM chunks WHERE x = :x AND z = :z").unwrap();
                    //conn.blob_open(DatabaseName::Main, "chunks", "block_data", row, true)
                    let compressed_block_data: Vec<u8> = load_stmt.query_row(
                        &[&coord.x, &coord.z],
                        |row| row.get(0)
                    ).unwrap();
                    let block_data = inflate_bytes_zlib(&compressed_block_data).unwrap();
                    let chunk = Chunk::from_bytes(&block_data);
                    tx.send((coord, chunk)).unwrap();
                }
                Request::Save(coord, chunk) => {
                    let mut store_stmt = trans.prepare_cached("INSERT OR REPLACE INTO chunks (x, z, block_data) VALUES (:x, :z, :block_data)").unwrap();

                    let block_data = chunk.to_bytes();
                    let compressed_block_data = deflate_bytes_zlib(&block_data);
                    store_stmt.execute_named(&[
                        (":x", &coord.x),
                        (":z", &coord.z),
                        (":block_data", &compressed_block_data)
                    ]).unwrap();
                }
                Request::Close => unreachable!(),
            }
        }
        trans.commit().unwrap();
    }
}

pub struct ResponseIter<'a>(mpsc::TryIter<'a, (ChunkCoord, Box<Chunk>)>);

impl<'a> Iterator for ResponseIter<'a> {
    type Item = (ChunkCoord, Box<Chunk>);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
