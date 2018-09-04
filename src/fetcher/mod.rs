use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use failure;
use futures;
use futures::prelude::*;
use hyper;
use hyper_tls;
use url::Url;

mod cache;
use self::cache::{Cache, CacheItem};

pub struct Fetcher {
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    cache: Arc<Mutex<Cache>>,
}

impl Fetcher {
    pub fn new() -> Result<Fetcher, failure::Error> {
        let https = hyper_tls::HttpsConnector::new(4)?;
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let fetcher = Fetcher {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
        };
        Ok(fetcher)
    }

    pub fn get(
        &mut self,
        uri: String,
    ) -> Box<dyn Future<Item = hyper::Chunk, Error = failure::Error> + Send> {
        let scheme_is_file = uri.starts_with("file:")
            || uri.starts_with("/")
            || uri.starts_with("./")
            || uri.starts_with("../");
        let scheme_is_http = uri.starts_with("http://") || uri.starts_with("https://");
        if scheme_is_file {
            let path;
            if uri.starts_with("file:") {
                // TODO Error handling
                let url = Url::parse(&uri).unwrap();
                path = url.to_file_path().unwrap();
            } else {
                path = PathBuf::from(uri);
            }
            Box::new(self.get_file(path))
        } else if scheme_is_http {
            Box::new(self.get_http(uri.parse().unwrap()))
        } else {
            Box::new(futures::future::err(format_err!("Invalid URI: {}", uri)))
        }
    }

    fn get_file(&self, path: PathBuf) -> impl Future<Item = hyper::Chunk, Error = failure::Error> {
        futures::lazy(move || {
            let mut file = File::open(path)?;
            let mut file_contents = vec![];
            file.read_to_end(&mut file_contents)?;
            let chunk = hyper::Chunk::from(file_contents);
            Ok(chunk)
        })
    }

    fn get_http(
        &mut self,
        uri: hyper::Uri,
    ) -> impl Future<Item = hyper::Chunk, Error = failure::Error> {
        let cache = self.cache.clone();
        let cached_item = futures::lazy({
            let cache = cache.clone();
            let uri = uri.clone();
            move || {
                let mut lock = cache.lock().map_err(|_| ())?;
                lock.get(&uri)
                    .map(|item| hyper::Chunk::from(item.data))
                    .ok_or(())
            }
        });
        let fetch = self
            .client
            .get(uri.clone())
            .from_err()
            .and_then(|response| {
                if response.status() != hyper::StatusCode::OK {
                    bail!("HTTP status code: {}", response.status())
                } else {
                    Ok(response.into_body().concat2().from_err())
                }
            }).flatten()
            .and_then(move |chunk| {
                cache
                    .lock()
                    .map_err(|_| format_err!("cache locking failed"))?
                    .put(uri, CacheItem::new(chunk.to_vec(), ()));
                Ok(chunk)
            });
        cached_item.or_else(|_| fetch)
    }
}
