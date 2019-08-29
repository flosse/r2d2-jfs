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
//! fn main() {
//!     let manager = JfsConnectionManager::file("file.json");
//!     let pool = r2d2::Pool::builder().max_size(1).build(manager).unwrap();
//!     let mut threads = vec![];
//!     for i in 0..10 {
//!         let pool = pool.clone();
//!         threads.push(thread::spawn(move || {
//!             let d = Data { x: i };
//!             let conn = pool.get().unwrap();
//!             conn.save(&d).unwrap();
//!         }));
//!     }
//!     for c in threads {
//!         c.join().unwrap();
//!     }
//! }
//! ```

use jfs::{self, Store, IN_MEMORY};
use std::{
    io,
    path::{Path, PathBuf},
};

enum Config {
    File(PathBuf, jfs::Config),
    Memory(Store),
}

/// An `r2d2::ManageConnection` for `jfs::Store`s.
pub struct JfsConnectionManager(Config);

impl JfsConnectionManager {
    /// Creates a new `JfsConnectionManager` for a single json file.
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        let mut cfg = jfs::Config::default();
        cfg.single = true;
        Self(Config::File(path.as_ref().into(), cfg))
    }
    /// Creates a new `JfsConnectionManager` for a directory with json files.
    pub fn dir<P: AsRef<Path>>(path: P) -> Self {
        let mut cfg = jfs::Config::default();
        cfg.single = false;
        Self(Config::File(path.as_ref().into(), cfg))
    }
    /// Creates a new `JfsConnectionManager` with a in-memory store.
    pub fn memory() -> Self {
        Self(Config::Memory(
            Store::new(IN_MEMORY).expect("Unable to initialize in-memory store"),
        ))
    }

    /// Creates a new `JfsConnectionManager` with the given path and jfs::Config
    pub fn new_with_cfg<P: AsRef<Path>>(path: P, cfg: jfs::Config) -> Self {
        Self(Config::File(path.as_ref().into(), cfg))
    }
}

impl r2d2::ManageConnection for JfsConnectionManager {
    type Connection = jfs::Store;
    type Error = io::Error;

    fn connect(&self) -> Result<Store, Self::Error> {
        match &self.0 {
            Config::File(path, cfg) => Store::new_with_cfg(path, *cfg),
            Config::Memory(store) => Ok(store.clone()),
        }
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}
