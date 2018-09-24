use std::io::Cursor;

use bytes::Bytes;
use futures::sync::mpsc::UnboundedSender;
use html5ever::parse_document;
use html5ever::rcdom::{Node, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use std::collections::HashMap;
use url::Url;

use crate::css::Stylesheet;
use crate::fetcher::{self, Data};
use crate::fetcher::thread::{FetcherRequest, make_request};

pub struct Page {
    pub url: Url,
    pub resources: Resources,
    pub dom: RcDom,
    pub stylesheets: Vec<Stylesheet>,
}

pub fn fetch(
    url: Url,
    request_tx: &UnboundedSender<FetcherRequest>,
) -> Result<Page, failure::Error> {
    let (url, page) = make_request(url, request_tx).wait();
    let page = page?;
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut Cursor::new(page))?;
    let mut requests: Vec<fetcher::thread::Receiver> = vec![];
    fetch_resources_from_dom(
        &url,
        &dom.document,
        request_tx,
        &mut requests,
        FetchState::default(),
    );
    let mut resources = Resources::default();
    for request in requests {
        let (url, response) = request.wait();
        let (parts, data) = match response? {
            Data::File(data) => (None, data),
            Data::Http(response) => {
                let (parts, data) = response.into_parts();
                (Some(parts), data)
            }
        };
        resources
            .resources
            .insert(url.clone(), Resource { url, parts, data });
    }
    Ok(Page {
        url,
        resources,
        dom,
        stylesheets: vec![],
    })
}

fn fetch_resources_from_dom(
    origin: &Url,
    node: &Node,
    tx: &UnboundedSender<FetcherRequest>,
    requests: &mut Vec<fetcher::thread::Receiver>,
    mut fetch_state: FetchState,
) {
    match node.data {
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => match &*name.local.to_ascii_lowercase() {
            "head" => {
                fetch_state = FetchState::InsideHead;
            }
            "body" => {
                fetch_state = FetchState::InsideBody;
            }
            "link" => {
                let attrs = attrs.borrow();
                if attrs
                    .iter()
                    .any(|attr| &attr.name.local == "rel" && &*attr.value == "stylesheet")
                {
                    if let Some(url) = attrs.iter().find(|attr| &attr.name.local == "href") {
                        if let Ok(url) = origin.join(&url.value) {
                            requests.push(make_request(url, tx));
                        }
                    }
                }
            }
            "img" => {
                let attrs = attrs.borrow();
                if let Some(url) = attrs.iter().find(|attr| &attr.name.local == "src") {
                    if let Ok(url) = origin.join(&url.value) {
                        requests.push(make_request(url, tx));
                    }
                }
            }
            "template" => {
                return;
            }
            _ => {}
        },
        _ => {}
    }
    for child in node.children.borrow().iter() {
        fetch_resources_from_dom(origin, child, tx, requests, fetch_state);
    }
}

#[derive(Copy, Clone)]
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
