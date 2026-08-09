use super::error::DbError;
use super::migrations;

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct SqliteHandle {
    conn: Mutex<Connection>,
}

impl SqliteHandle {
    pub fn new(file_path: impl AsRef<Path>) -> Result<Self, DbError> {
        let mut conn = Connection::open(file_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA synchronous = NORMAL;"
        )?;

        migrations::migrations().to_latest(&mut conn)?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn with_connection<T>(&self, f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Result<T, DbError> {
        let conn = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        f(&conn).map_err(DbError::from)
    }
}