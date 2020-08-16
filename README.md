# Redis Cache

A shared memory cache for casbin-rs, backed by redis.

## Installation

```rust
[dependencies]
casbin = { git = "https://github.com/casbin/casbin-rs", branch = "release-v2.0.0", default-features = false, features = ["runtime-async-std", "logging", "cached"] }
redis-cache = { git = "https://github.com/casbin-rs/redis-cache" }
async-std = { version = "1.5.0", features = ["attributes"] }
env_logger = "0.7.1"
```

## Getting started

```rust
use casbin::prelude::*;
use redis_cache::RedisCache;

#[async_std::main]
async fn main() -> Result<()> {
    ::std::env::set_var("RUST_LOG", "casbin=info");
    env_logger::init();

    let mut e = CachedEnforcer::new("examples/rbac_with_domains_model.conf", "examples/rbac_with_domains_policy.csv").await?;
    e.enable_log(true);

    let redis_cache: RedisCache<Vec<String>, bool> = RedisCache::new("redis://localhost:6379/1");

    e.set_cache(Box::new(redis_cache));
    e.enforce(&["alice", "domain1", "data1", "read"]).await?;
}
```
