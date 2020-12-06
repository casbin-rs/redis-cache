use casbin::Cache;

use redis::{Client, Connection, IntoConnectionInfo, RedisResult};
use serde::{de::DeserializeOwned, Serialize};

use std::{borrow::Cow, hash::Hash, marker::PhantomData};

const CACHE_HKEY: &str = "casbin_cache";

pub struct RedisCache<K, V> {
    conn: Connection,
    cap: usize,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> RedisCache<K, V> {
    pub fn new<U: IntoConnectionInfo>(url: U) -> RedisResult<RedisCache<K, V>> {
        let client = Client::open(url)?;
        let conn = client.get_connection()?;

        Ok(RedisCache {
            conn,
            cap: 1000,
            _marker: PhantomData,
        })
    }
}

impl<K, V> Cache<K, V> for RedisCache<K, V>
where
    K: Eq + Hash + Send + Sync + Serialize + 'static,
    V: Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    fn set_capacity(&mut self, cap: usize) {
        self.cap = cap;
    }

    fn get(&mut self, k: &K) -> Option<Cow<'_, V>> {
        if let Ok(field) = serde_json::to_string(&k) {
            if let Ok(res) = redis::cmd("HGET")
                .arg(CACHE_HKEY)
                .arg(field)
                .query::<String>(&mut self.conn)
            {
                return serde_json::from_str(&res).map(|x| Cow::Owned(x)).ok();
            }
        }

        None
    }

    fn has(&mut self, k: &K) -> bool {
        if let Ok(field) = serde_json::to_string(&k) {
            if let Ok(res) = redis::cmd("HEXISTS")
                .arg(CACHE_HKEY)
                .arg(field)
                .query::<bool>(&mut self.conn)
            {
                return res;
            }
        }

        false
    }

    fn set(&mut self, k: K, v: V) {
        if let Ok(keys) = redis::cmd("HKEYS")
            .arg(CACHE_HKEY)
            .query::<Vec<String>>(&mut self.conn)
        {
            if keys.len() < self.cap {
                if let (Ok(field), Ok(value)) =
                    (serde_json::to_string(&k), serde_json::to_string(&v))
                {
                    let _ = redis::cmd("HSET")
                        .arg(CACHE_HKEY)
                        .arg(field)
                        .arg(value)
                        .query::<bool>(&mut self.conn);
                }
            }
        }
    }

    fn clear(&mut self) {
        let _ = redis::cmd("DEL")
            .arg(CACHE_HKEY)
            .query::<bool>(&mut self.conn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_has_get_clear() {
        let mut cache = RedisCache::new("redis://localhost:6379").unwrap();

        cache.set(vec!["alice", "/data1", "read"], false);
        assert!(cache.has(&vec!["alice", "/data1", "read"]));
        assert!(cache.get(&vec!["alice", "/data1", "read"]) == Some(Cow::Borrowed(&false)));
        cache.clear();

        assert!(cache.get(&vec!["alice", "/data1", "read"]) == None);
    }
}
