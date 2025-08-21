In‑Memory Configuration Store Powered by Serde
Version 1.0 – August 2025

# 1. Purpose & Scope
This document describes the design and implementation of a typed, in‑memory key/value configuration subsystem for the Acme Rust application. The subsystem must:

Requirement	Why it matters
Load defaults from a file (JSON/TOML/YAML) at start‑up	Allows operators to ship a single source of truth that can be edited with familiar tools.
Provide type‑safe read access (cfg.get::<bool>("debug")) throughout the code base	Prevents runtime parsing errors and eliminates boilerplate match/parse.
Permit runtime overrides (e.g., command‑line flags, hot‑reload) without touching the original file.	
Be readable from any thread with minimal contention.	
Remain dependency‑light – only Serde and a tiny runtime crate (once_cell).	
The design below satisfies all of these goals while staying idiomatic to modern Rust.

# 2. High‑Level Overview

```
+-------------------+        +----------------------+        +-----------------+
| Config File (JSON | ----> |   Deserialization    | ----> |  ConfigStore     |
| / TOML / YAML)    |        |   (serde_json, etc.)|        | (HashMap<String,|
+-------------------+        +----------------------+        |  ConfigValue>) |
                                                               ^   |
                                                               |   |
               +-----------------------------------------------+   |
               |                                                   |
               v                                                   |
          Global Singleton (once_cell::Lazy)                        |
               |                                                   |
    +----------+-----------+                               +-------+--------+
    |                      |                               |                |
    v                      v                               v                v
App modules            Background task                HTTP handler      CLI command
(read‑only)             (hot‑reload)                 (read config)   (override)
```

ConfigFile – any format supported by Serde (json, toml, yaml).
Deserialization Layer – a thin conversion from the generic `serde_json::Value` (or equivalent) into our own ConfigValue enum.
ConfigStore – the in‑memory container that owns a `HashMap<String, ConfigValue>`. All reads go through this store.
Global Singleton – created once at program start via once_cell::sync::Lazy. The singleton can be wrapped in an `Arc<RwLock<_>>` if mutable access from many threads is required.

# 3. Component Specification

## 3.1 ConfigValue

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),

    // Optional extensions – keep them disabled until needed.
    #[serde(skip_serializing_if = "Option::is_none")]
    Array(Option<Vec<ConfigValue>>),

    #[serde(skip_serializing_if = "Option::is_none")]
    Map(Option<Box<HashMap<String, ConfigValue>>>),
}
```

All variants are #[derive(Serialize, Deserialize)] so the enum can be round‑tripped if the application ever needs to write a config file back to disk.

## 3.2 ConfigStore

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConfigStore {
    map: HashMap<String, ConfigValue>,
}

Public API
Method	Signature	Description
new()	fn new() -> Self	Empty store – used mainly in tests.
from_serde_value(root: impl serde_json::Value)	Self	Convert a deserialized JSON/TOML/YAML value into a flattened store (top‑level keys become map entries).
set<V>(&mut self, key: impl Into<String>, val: V)	where V: Into<ConfigValue>	Insert or replace a configuration entry at runtime.
get<T>(&self, key: &str) -> Option<T>	where T: ConfigExtract	Typed read – returns None if the key is missing or cannot be converted to T.
merge(&mut self, other: Self)	–	Utility for hot‑reload: overlay a new store on top of the existing one.
3.3 Extraction Trait (ConfigExtract)
pub trait ConfigExtract: Sized {
    fn extract(v: &ConfigValue) -> Option<Self>;
}

Implementations exist for bool, i64, f64, String. Adding a new type (e.g., Vec<String>) only requires adding another impl – no changes to the store.

3.4 Global Access (CONFIG)
use once_cell::sync::Lazy;

static CONFIG: Lazy<ConfigStore> = Lazy::new(|| {
    // 1️⃣ Load file (JSON shown, swap for toml/yaml as needed)
    let raw = std::fs::read_to_string("config.json")
        .expect("Failed to read config.json");

    // 2️⃣ Parse into a generic serde value
    let root: serde_json::Value =
        serde_json::from_str(&raw).expect("Invalid JSON configuration");

    // 3️⃣ Convert to our typed store
    ConfigStore::from_serde_value(root)
});
```

The Lazy wrapper guarantees thread‑safe, one‑time initialization.

If mutable access from many threads is required (e.g., hot‑reload), replace the definition with:

```rust
use std::sync::{Arc, RwLock};

static CONFIG: Lazy<Arc<RwLock<ConfigStore>>> = Lazy::new(|| {
    Arc::new(RwLock::new(/* same init as above */))
});
```

# 4. Runtime Scenarios

## 4.1 Simple Typed Read (any module)

```rust
fn start_server() {
    // No need to import CONFIG; it is a global.
    let port: i64 = CONFIG.get("http_port").unwrap_or(8080);
    let log_level: String = CONFIG.get("log.level")
        .unwrap_or_else(|| "info".into());

    println!("Listening on 0.0.0.0:{port} (log={log_level})");
}
```

## 4.2 Overriding via CLI Flags

```rust
fn apply_cli_overrides(matches: &clap::ArgMatches) {
    // Obtain a mutable copy – we don’t want to lock the global store for the whole app.
    let mut cfg = CONFIG.clone();

    if let Some(port) = matches.value_of_t::<i64>("port").ok() {
        cfg.set("http_port", port);
    }
    if matches.is_present("debug") {
        cfg.set("log.level", "debug");
    }

    // Replace the global (if using Arc<RwLock>)
    #[cfg(feature = "concurrent")]
    *CONFIG.write().unwrap() = cfg;
}
```

## 4.3 Hot‑Reload from Disk

```rust
use std::time::{Duration, Instant};
use std::thread;

fn spawn_hot_reload_watcher(path: &'static str) {
    thread::spawn(move || {
        let mut last_mod = std::fs::metadata(path).unwrap().modified().unwrap();

        loop {
            thread::sleep(Duration::from_secs(5));
            if let Ok(meta) = std::fs::metadata(path) {
                if let Ok(modified) = meta.modified() {
                    if modified > last_mod {
                        // Load the new file
                        let raw = std::fs::read_to_string(path).unwrap();
                        let root: serde_json::Value =
                            serde_json::from_str(&raw).expect("bad config");
                        let new_store = ConfigStore::from_serde_value(root);

                        // Merge – new values win, old ones stay if missing
                        #[cfg(feature = "concurrent")]
                        {
                            let mut guard = CONFIG.write().unwrap();
                            guard.merge(new_store);
                        }
                        #[cfg(not(feature = "concurrent"))]
                        {
                            CONFIG = Lazy::new(|| new_store); // replace whole store
                        }

                        last_mod = modified;
                        println!("[watcher] configuration reloaded");
                    }
                }
            }
        }
    });
}
```

The watcher runs in a background thread, checks the file timestamp every 5 seconds, and merges changes atomically.

# 5. Extensibility Guidelines

Change	Impact on Existing Code

Add a new primitive type (e.g., u32)	Add a variant to ConfigValue, an impl From<u32> and a ConfigExtract impl – no other changes.
Support nested tables	Use the existing Map(Box<HashMap<…>>) variant; provide helper methods like fn get_map(&self, key: &str) -> Option<&HashMap<String, ConfigValue>>.
Switch file format (e.g., from JSON to TOML)	Replace the call to serde_json::from_str with toml::from_str; keep the rest of the pipeline unchanged because both produce a serde_json::Value‑compatible structure via serde::de::DeserializeOwned.
Persist changes back to disk	Because ConfigValue is Serde‑serializable, implement fn write_to(path: &Path) -> Result<()> that calls serde_json::to_string_pretty(&self.map) (or the appropriate format).

# 6. Non‑Functional Considerations

Aspect	Decision

Performance	Look‑ups are O(1) hash map accesses; conversion from ConfigValue to concrete types is a cheap match.
Memory Footprint	Each entry stores the key string + enum (average 24 bytes per primitive). For typical app configs (< 200 entries) this is negligible.
Thread‑Safety	By default the store is immutable after start‑up; for mutable scenarios we recommend Arc<RwLock<ConfigStore>>. The lock is only taken on writes (hot‑reload, CLI overrides), which are rare.
Error Handling	All deserialization errors cause a panic at start‑up – this is intentional because the application cannot run without a valid config. Runtime get<T> returns Option<T>; callers decide whether to fallback to defaults.
Testing	The store can be instantiated directly (ConfigStore::new()) and populated with test data, making unit tests straightforward.

# 7. Example Project Layout

```txt
src/
│
├─ config/
│   ├─ mod.rs          // re‑exports ConfigStore, CONFIG, traits
│   ├─ value.rs        // definition of ConfigValue + serde impls
│   └─ store.rs        // implementation of ConfigStore & extraction trait
│
├─ main.rs             // creates the global CONFIG via once_cell
└─ lib.rs              // optional shared library entry point
```


config/mod.rs (excerpt)

```rust
pub mod value;
pub mod store;

pub use store::{ConfigStore, ConfigExtract};
pub use crate::config::value::ConfigValue;

// Global singleton – compiled with the `concurrent` feature if needed.
#[cfg(not(feature = "concurrent"))]
pub static CONFIG: once_cell::sync::Lazy<ConfigStore> = once_cell::sync::Lazy::new(|| {
    // same init as in Section 3.4
});

#[cfg(feature = "concurrent")]
pub static CONFIG: once_cell::sync::Lazy<std::sync::Arc<std::sync::RwLock<ConfigStore>>> =
    once_cell::sync::Lazy::new(|| {
        std::sync::Arc::new(std::sync::RwLock::new(/* init */))
    });
```

# 8. References
Serde – 
https://serde.rs/

 (de/serialization framework)
once_cell – 
https://crates.io/crates/once_cell

 (lazy statics)
DashMap – alternative if lock‑free concurrency is required (outside the scope of this doc).