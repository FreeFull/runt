use hyper;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Cache {
    cache: HashMap<hyper::Uri, CacheItem>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, url: &hyper::Uri) -> Option<CacheItem> {
        let item = self.cache.get(url).cloned();
        if let Some(item) = item {
            if item.expired() {
                self.cache.remove(url);
                None
            } else {
                Some(item)
            }
        } else {
            None
        }
    }

    pub fn put(&mut self, url: hyper::Uri, item: CacheItem) {
        self.cache.insert(url, item);
        self.clean_up();
        println!("Cache: {:?}", self);
    }

    pub fn clean_up(&mut self) {
        self.cache.retain(|_key, value| {
            !value.expired()
        });
    }
}

#[derive(Clone, Debug)]
pub struct CacheItem {
    pub data: Vec<u8>,
    pub expires: (), // TODO
}

impl CacheItem {
    pub fn new(data: Vec<u8>, expiration: ()) -> CacheItem {
        CacheItem {
            data,
            expires: expiration,
        }
    }

    // TODO
    pub fn expired(&self) -> bool {
        false
    }
}
