use futures::prelude::*;

use failure;

use hyper;
use hyper_tls;

mod cache;
use self::cache::Cache;

pub struct Fetcher {
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    cache: Cache,
}

impl Fetcher {
    pub fn new() -> Result<Fetcher, failure::Error> {
        let https = hyper_tls::HttpsConnector::new(4)?;
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let fetcher = Fetcher {
            client,
            cache: Cache::new(),
        };
        Ok(fetcher)
    }

    pub fn get(
        &mut self,
        uri: hyper::Uri,
    ) -> impl Future<Item = hyper::Chunk, Error = failure::Error> {
        self.client
            .get(uri)
            .from_err()
            .and_then(|response| {
                if response.status() != hyper::StatusCode::OK {
                    bail!("HTTP status code: {}", response.status())
                } else {
                    Ok(response.into_body().concat2().from_err())
                }
            }).and_then(|chunk| chunk)
    }
}
