//! # r2d2-jfs
//!
//! [JSON file store (`jfs`)](https://crates.io/crates/jfs) support for the
//! [`r2d2`](https://crates.io/crates/r2d2) connection pool.
//!
//! ## Example
//!
//! ```rust,no_run
//! use std::thread;
//! use serde::{Serialize, Deserialize};
//! use r2d2_jfs::JfsConnectionManager;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Data { x: i32 }
//!
//!let manager = JfsConnectionManager::file("file.json").unwrap();
//!let pool = r2d2::Pool::builder().max_size(5).build(manager).unwrap();
//!let mut threads = vec![];
//!for i in 0..10 {
//!    let pool = pool.clone();
//!    threads.push(thread::spawn(move || {
//!        let d = Data { x: i };
//!        let conn = pool.get().unwrap();
//!        conn.save(&d).unwrap();
//!    }));
//!}
//!for c in threads {
//!    c.join().unwrap();
//!}
//! ```

use jfs::{self, Store, IN_MEMORY};
use std::{io, path::Path};

/// An `r2d2::ManageConnection` for `jfs::Store`s.
pub struct JfsConnectionManager(Store);

impl JfsConnectionManager {
    /// Creates a new `JfsConnectionManager` for a single json file.
    pub fn file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let cfg = jfs::Config {
            single: true,
            ..Default::default()
        };
        Self::new_with_cfg(path, cfg)
    }
    /// Creates a new `JfsConnectionManager` for a directory with json files.
    pub fn dir<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let cfg = jfs::Config {
            single: false,
            ..Default::default()
        };
        Self::new_with_cfg(path, cfg)
    }
    /// Creates a new `JfsConnectionManager` with a in-memory store.
    pub fn memory() -> Self {
        Self(Store::new(IN_MEMORY).expect("Unable to initialize in-memory store"))
    }

    /// Creates a new `JfsConnectionManager` with the given path and jfs::Config
    pub fn new_with_cfg<P: AsRef<Path>>(path: P, cfg: jfs::Config) -> io::Result<Self> {
        let store = Store::new_with_cfg(path, cfg)?;
        Ok(Self(store))
    }
}

impl r2d2::ManageConnection for JfsConnectionManager {
    type Connection = jfs::Store;
    type Error = io::Error;

    fn connect(&self) -> Result<Store, Self::Error> {
        Ok(self.0.clone())
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::thread;
    use tempdir::TempDir;

    #[test]
    fn multi_threading() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            x: i32,
        }
        let dir = TempDir::new("r2d2-jfs-test").expect("Could not create temporary directory");
        let file = dir.path().join("db.json");
        let manager = JfsConnectionManager::file(file).unwrap();
        let pool = r2d2::Pool::builder().max_size(5).build(manager).unwrap();
        let mut threads: Vec<thread::JoinHandle<()>> = vec![];
        for i in 0..20 {
            let pool = pool.clone();
            let x = Data { x: i };
            threads.push(thread::spawn(move || {
                let db = pool.get().unwrap();
                db.save_with_id(&x, &i.to_string()).unwrap();
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
        let db = pool.get().unwrap();
        let all = db.all::<Data>().unwrap();
        assert_eq!(all.len(), 20);
        for (id, data) in all {
            assert_eq!(data.x.to_string(), id);
        }
    }
}
