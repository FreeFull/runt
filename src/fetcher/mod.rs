use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::format_err;
use bytes::Bytes;
use url::Url;

mod cache;
use self::cache::Cache;

#[derive(Debug, Clone)]
pub struct Fetcher {
    client:
        hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
    cache: Arc<Mutex<Cache>>,
}

impl Fetcher {
    pub fn new() -> Result<Fetcher, Error> {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let fetcher = Fetcher {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
        };
        Ok(fetcher)
    }

    pub async fn get(&self, url: &Url) -> Result<Data, Error> {
        let scheme_is_file = url.scheme() == "file";
        let scheme_is_http = url.scheme() == "http" || url.scheme() == "https";
        if scheme_is_file {
            let path = url
                .to_file_path()
                .map_err(|()| format_err!("Failed to convert `file:` URI to a path: {}", url));
            match path {
                Ok(path) => self.get_file(path).map(Data::File),
                Err(err) => Err(err.into()),
            }
        } else if scheme_is_http {
            // It is theoretically possible that a valid url::Url isn't a valid hyper::Uri
            // For now, take the easiest route for converting between the two
            let hyper_uri = url.as_ref().parse().unwrap();
            Ok(Data::Http(self.get_http(hyper_uri).await?))
        } else {
            Err(format_err!("Invalid URL: {}", url).into())
        }
    }

    pub async fn get_with_redirect(
        &mut self,
        mut url: Url,
        max_redirects: u16,
    ) -> Result<Data, Error> {
        let fetcher = self;
        let data = fetcher.get(&url).await?;
        for i in 0..max_redirects {
            match data {
                Data::File(_) => return Ok(data),
                Data::Http(ref request) => {
                    if request.status().is_redirection() && max_redirects > 0 {
                        if let Some(new_url) = request.headers().get(hyper::header::LOCATION) {
                            url = url.join(new_url.to_str().map_err(anyhow::Error::from)?)?;
                        } else {
                            return Ok(data);
                        }
                    } else {
                        return Ok(data);
                    }
                }
            }
        }
        Ok(data)
    }

    fn get_file(&self, path: PathBuf) -> Result<Bytes, Error> {
        let mut file = File::open(path)?;
        let mut file_contents = vec![];
        file.read_to_end(&mut file_contents)?;
        let bytes = Bytes::from(file_contents);
        Ok(bytes)
    }

    async fn get_http(&self, uri: hyper::Uri) -> Result<hyper::Response<Bytes>, Error> {
        let response = self.client.get(uri.clone()).await?;
        let (parts, body) = response.into_parts();
        Ok(hyper::Response::from_parts(
            parts,
            hyper::body::to_bytes(body).await?,
        ))
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
    Http(hyper::Error),
    UrlParseError(url::ParseError),
    Other(anyhow::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::File(ref err) => err.fmt(f),
            Error::Http(ref err) => err.fmt(f),
            Error::UrlParseError(ref err) => err.fmt(f),
            Error::Other(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::File(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::Http(error)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Error {
        Error::UrlParseError(error)
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Error {
        Error::Other(error)
    }
}

impl std::error::Error for Error {}
