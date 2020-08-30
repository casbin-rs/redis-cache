use casbin::Cache;

use redis::{Client, Connection, IntoConnectionInfo, RedisResult};
use serde::{de::DeserializeOwned, Serialize};

use std::{borrow::Cow, hash::Hash, marker::PhantomData};

const CACHE_PREFIX: &str = "casbin";

pub struct RedisCache<K, V>
where
    K: Eq + Hash + Send + Sync + Serialize + 'static,
    V: Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    conn: Connection,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> RedisCache<K, V>
where
    K: Eq + Hash + Send + Sync + Serialize + 'static,
    V: Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    pub fn new<U: IntoConnectionInfo>(url: U) -> RedisResult<RedisCache<K, V>> {
        let client = Client::open(url)?;
        let conn = client.get_connection()?;

        Ok(RedisCache {
            conn,
            _marker: PhantomData,
        })
    }
}

impl<K, V> Cache<K, V> for RedisCache<K, V>
where
    K: Eq + Hash + Send + Sync + Serialize + 'static,
    V: Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    // This function shouldn't be implemented, users can configure redis maxmium memory & lru
    // elimination strategy to achive lru-cache similar effect
    fn set_capacity(&mut self, _c: usize) {}

    fn get(&mut self, k: &K) -> Option<Cow<'_, V>> {
        if let Ok(ser_key) = serde_json::to_string(&k) {
            let cache_key = format!("{}::{}", CACHE_PREFIX, ser_key);
            if let Ok(res) = redis::cmd("GET")
                .arg(cache_key)
                .query::<String>(&mut self.conn)
            {
                return serde_json::from_str(&res).ok().map(|x| Cow::Owned(x));
            }
        }

        None
    }

    fn has(&mut self, k: &K) -> bool {
        self.get(k).is_some()
    }

    fn set(&mut self, k: K, v: V) {
        if let (Ok(ser_key), Ok(ser_val)) = (serde_json::to_string(&k), serde_json::to_string(&v)) {
            let cache_key = format!("{}::{}", CACHE_PREFIX, ser_key);
            let _ = redis::cmd("SET")
                .arg(cache_key)
                .arg(ser_val)
                .query::<String>(&mut self.conn);
        }
    }

    fn clear(&mut self) {
        let cache_key = format!("{}::*", CACHE_PREFIX);

        let script = format!(
            r#"
            EVAL "for i, name in ipairs(redis.call('KEYS', '{}')) do redis.call('del', name, 0); end" 0
        "#,
            cache_key
        );

        let script = redis::Script::new(&script);
        let _ = script
            .arg(1)
            .arg(2)
            .invoke::<Option<String>>(&mut self.conn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut cache = RedisCache::new("redis://localhost:6379").unwrap();

        cache.set(vec!["alice", "/data1", "read"], false);
        assert!(cache.has(&vec!["alice", "/data1", "read"]));
        assert!(cache.get(&vec!["alice", "/data1", "read"]) == Some(Cow::Borrowed(&false)));
    }

    #[test]
    fn test_clear() {
        let mut cache = RedisCache::new("redis://localhost:6379").unwrap();

        cache.set(vec!["alice", "/data1", "read"], false);
        cache.clear();
        assert!(cache.get(&vec!["alice", "/data1", "read"]) == None);
    }
}
