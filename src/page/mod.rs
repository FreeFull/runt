use std;
use std::io::Cursor;
use std::thread;

use bytes::Bytes;
use failure;
use futures::sync::mpsc::{unbounded, UnboundedSender};
use futures::{self, Future, Sink, Stream};
use html5ever::parse_document;
use html5ever::rcdom::{Node, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use http;
use std::collections::HashMap;
use tokio;
use url::Url;

use css::Stylesheet;
use fetcher::{Fetcher, Data};

pub struct Page {
    pub url: Url,
    pub resources: Resources,
    pub dom: RcDom,
    pub stylesheets: Vec<Stylesheet>,
}

pub fn fetch(url: Url) -> Result<Page, failure::Error> {
    let (request_tx, request_rx) = unbounded();
    let (results_tx, results_rx) = unbounded();
    thread::spawn(move || {
        tokio::run(
            futures::lazy(move || {
                let mut fetcher = Fetcher::new().unwrap();
                request_rx
                    .and_then(move |url: Url| {
                        fetcher.get(&url).then(move |response| Ok((url, response)))
                    }).forward(results_tx.sink_map_err(|_| ()))
            }).then(|_| Ok(())),
        )
    });
    request_tx.unbounded_send(url.clone()).unwrap();
    let mut results_iter = results_rx.wait();
    let (url, page) = results_iter.next().unwrap().unwrap();
    let page = page?;
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut Cursor::new(page))?;
    fetch_resources_from_dom(&dom.document, &request_tx, FetchState::default());
    drop(request_tx);
    let mut resources = Resources::default();
    for result in results_iter {
        let (url, response) = result.unwrap();
        let (parts, data) = match response? {
            Data::File(data) => (None, data),
            Data::Http(response) => {
                let (parts, data) = response.into_parts();
                (Some(parts), data)
            }
        };
        resources.resources.insert(
            url.clone(),
            Resource {
                url,
                parts,
                data,
            },
        );
    }
    Ok(Page {
        url,
        resources,
        dom,
        stylesheets: vec![],
    })
}

// TODO
fn fetch_resources_from_dom(node: &Node, tx: &UnboundedSender<Url>, mut fetch_flags: FetchState) {
    match node.data {
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => match &*name.local.to_ascii_lowercase() {
            "head" => {}
            "body" => {}
            "link" => {}
            "img" => {}
            _ => {}
        },
        _ => {}
    }
}

enum FetchState {
    InsideHead,
    InsideBody,
    Default,
}

impl std::default::Default for FetchState {
    fn default() -> FetchState {
        FetchState::Default
    }
}

#[derive(Debug, Default)]
pub struct Resources {
    resources: HashMap<Url, Resource>,
}

#[derive(Debug)]
pub struct Resource {
    url: Url,
    parts: Option<http::response::Parts>,
    data: Bytes,
}
