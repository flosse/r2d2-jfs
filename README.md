# r2d2-jfs

## r2d2-jfs

[JSON file store (`jfs`)](https://crates.io/crates/jfs) support for the
[`r2d2`](https://crates.io/crates/r2d2) connection pool.

### Example

```rust
use std::thread;
use serde::{Serialize, Deserialize};
use r2d2_jfs::JfsConnectionManager;

#[derive(Serialize, Deserialize)]
struct Data { x: i32 }

fn main() {
    let manager = JfsConnectionManager::file("file.json");
    let pool = r2d2::Pool::builder().max_size(1).build(manager).unwrap();
    let mut threads = vec![];
    for i in 0..10 {
        let pool = pool.clone();
        threads.push(thread::spawn(move || {
            let d = Data { x: i };
            let conn = pool.get().unwrap();
            conn.save(&d).unwrap();
        }));
    }
    for c in threads {
        c.join().unwrap();
    }
}
```

License: Apache-2.0 OR MIT
