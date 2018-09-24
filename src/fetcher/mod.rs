use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use failure::{format_err, Fail};
use futures::prelude::*;
use url::Url;

mod cache;
use self::cache::Cache;
pub mod thread;


#[derive(Debug, Clone)]
pub struct Fetcher {
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    cache: Arc<Mutex<Cache>>,
}

impl Fetcher {
    pub fn new() -> Result<Fetcher, Error> {
        let https = hyper_tls::HttpsConnector::new(4)?;
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let fetcher = Fetcher {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
        };
        Ok(fetcher)
    }

    pub fn get(&self, url: &Url) -> Box<dyn Future<Item = Data, Error = Error> + Send> {
        let scheme_is_file = url.scheme() == "file";
        let scheme_is_http = url.scheme() == "http" || url.scheme() == "https";
        if scheme_is_file {
            let path = url
                .to_file_path()
                .map_err(|()| format_err!("Failed to convert `file:` URI to a path: {}", url));
            match path {
                Ok(path) => Box::new(self.get_file(path).map(Data::File)),
                Err(err) => Box::new(futures::future::err(err.into())),
            }
        } else if scheme_is_http {
            // It is theoretically possible that a valid url::Url isn't a valid hyper::Uri
            // For now, take the easiest route for converting between the two
            let hyper_uri = url.as_ref().parse().unwrap();
            Box::new(self.get_http(hyper_uri).map(Data::Http))
        } else {
            Box::new(futures::future::err(
                format_err!("Invalid URL: {}", url).into(),
            ))
        }
    }

    pub fn get_with_redirect(
        self,
        url: Url,
        max_redirects: u16,
    ) -> impl Future<Item = Data, Error = Error> {
        use futures::future::{loop_fn, Loop};
        let fetcher = self;
        loop_fn((url, max_redirects), move |(url, max_redirects)| {
            fetcher.get(&url).and_then(move |data| match data {
                Data::File(_) => Ok(Loop::Break(data)),
                Data::Http(ref request) => {
                    if request.status().is_redirection() && max_redirects > 0 {
                        if let Some(new_url) = request.headers().get(hyper::header::LOCATION) {
                            let url = url.join(new_url.to_str().map_err(failure::Error::from)?)?;
                            Ok(Loop::Continue((url, max_redirects - 1)))
                        } else {
                            Ok(Loop::Break(data))
                        }
                    } else {
                        Ok(Loop::Break(data))
                    }
                }
            })
        })
    }

    fn get_file(&self, path: PathBuf) -> impl Future<Item = Bytes, Error = Error> {
        futures::lazy(move || {
            let mut file = File::open(path)?;
            let mut file_contents = vec![];
            file.read_to_end(&mut file_contents)?;
            let bytes = Bytes::from(file_contents);
            Ok(bytes)
        })
    }

    fn get_http(
        &self,
        uri: hyper::Uri,
    ) -> impl Future<Item = hyper::Response<Bytes>, Error = Error> {
        self.client
            .get(uri.clone())
            .from_err()
            .and_then(|response| {
                let (parts, body) = response.into_parts();
                body.concat2().from_err().and_then(move |chunk| {
                    Ok(hyper::Response::from_parts(parts, chunk.into_bytes()))
                })
            })
    }
}

#[derive(Debug)]
pub enum Data {
    Http(hyper::Response<Bytes>),
    File(Bytes),
}

impl std::convert::AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        match *self {
            Data::Http(ref response) => response.body().as_ref(),
            Data::File(ref bytes) => bytes.as_ref(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    File(std::io::Error),
    Http(hyper::error::Error),
    Tls(hyper_tls::Error),
    UrlParseError(url::ParseError),
    Other(failure::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::File(ref err) => err.fmt(f),
            Error::Http(ref err) => err.fmt(f),
            Error::Tls(ref err) => err.fmt(f),
            Error::UrlParseError(ref err) => err.fmt(f),
            Error::Other(ref err) => err.fmt(f),
        }
    }
}

impl failure::Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        match *self {
            Error::File(ref err) => Some(err),
            Error::Http(ref err) => Some(err),
            Error::Tls(ref err) => Some(err),
            Error::UrlParseError(ref err) => Some(err),
            Error::Other(ref err) => Some(err.as_fail()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::File(error)
    }
}

impl From<hyper::error::Error> for Error {
    fn from(error: hyper::error::Error) -> Error {
        Error::Http(error)
    }
}

impl From<hyper_tls::Error> for Error {
    fn from(error: hyper_tls::Error) -> Error {
        Error::Tls(error)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Error {
        Error::UrlParseError(error)
    }
}

impl From<failure::Error> for Error {
    fn from(error: failure::Error) -> Error {
        Error::Other(error)
    }
}
