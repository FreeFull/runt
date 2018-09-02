use hyper;
use std::collections::HashMap;

pub struct Cache {
    cache: HashMap<hyper::Uri, CacheItem>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            cache: HashMap::new(),
        }
    }

    fn get(&mut self, url: &hyper::Uri) -> Option<CacheItem> {
        let item = self.cache.get(url);
        if let Some(item) = item {
            if item.expired() {
                None
            } else {
                Some(item.clone())
            }
        } else {
            None
        }
    }

    pub fn put(&mut self, url: hyper::Uri, item: CacheItem) {
        self.cache.insert(url, item);
    }

    pub fn clean_up(&mut self) {
        self.cache.retain(|_key, value| {
            !value.expired()
        });
    }
}

#[derive(Clone)]
pub struct CacheItem {
    pub data: Vec<u8>,
    pub expires: (), // TODO
}

impl CacheItem {
    // TODO
    fn expired(&self) -> bool {
        false
    }
}
