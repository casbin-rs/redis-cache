# Redis Cache

A shared memory cache for casbin-rs, backed by redis.

## Installation

```rust
[dependencies]
casbin = { version = "2.0.1", default-features = false, features = ["runtime-async-std", "logging", "cached"] }
redis-cache = { version = "0.1.0" }
async-std = { version = "1.5.0", features = ["attributes"] }
```

## Getting started

```rust
use casbin::prelude::*;
use redis_cache::RedisCache;

#[async_std::main]
async fn main() -> Result<()> {
    let mut e = CachedEnforcer::new("examples/rbac_with_domains_model.conf", "examples/rbac_with_domains_policy.csv").await?;
    e.enable_log(true);

    let redis_cache: RedisCache<u64, bool> = RedisCache::new("redis://localhost:6379/1");

    e.set_cache(Box::new(redis_cache));
    e.enforce_mut(("alice", "domain1", "data1", "read"))?;
}
```
