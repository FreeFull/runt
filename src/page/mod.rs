use std::io::Cursor;

use bytes::Bytes;
use futures::sync::mpsc::UnboundedSender;
use html5ever::parse_document;
use html5ever::rcdom::{Node, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use std::collections::HashMap;
use url::Url;

use crate::css::Stylesheet;
use crate::fetcher::thread::{make_request, FetcherRequest};
use crate::fetcher::{self, Data};

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
    let mut stylesheets = vec![];
    extract_stylesheets(&url, &dom.document, &resources, &mut stylesheets);
    Ok(Page {
        url,
        resources,
        dom,
        stylesheets,
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

fn extract_stylesheets(
    origin: &Url,
    node: &Node,
    resources: &Resources,
    stylesheets: &mut Vec<Stylesheet>,
) {
    match node.data {
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => match &*name.local.to_ascii_lowercase() {
            "link" => {
                let attrs = attrs.borrow();
                if attrs
                    .iter()
                    .any(|attr| &attr.name.local == "rel" && &*attr.value == "stylesheet")
                {
                    let url = match attrs.iter().find(|attr| &attr.name.local == "href") {
                        Some(url) => url,
                        None => return,
                    };
                    let url = match origin.join(&url.value) {
                        Ok(url) => url,
                        Err(_) => return,
                    };
                    if let Some(stylesheet) = resources.resources.get(&url).clone() {
                        if let Ok(stylesheet) = std::str::from_utf8(&stylesheet.data) {
                            // TODO: Error handling
                            if let Ok(stylesheet) = Stylesheet::parse(stylesheet, &url) {
                                stylesheets.push(stylesheet);
                            }
                        }
                    }
                }
            }
            "style" => {
                if let Some(text) = node.children.borrow().get(0) {
                    match text.data {
                        NodeData::Text { ref contents } => {
                            let stylesheet = Stylesheet::parse(&contents.borrow(), origin).unwrap();
                            stylesheets.push(stylesheet);
                        }
                        _ => return,
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
        extract_stylesheets(origin, child, resources, stylesheets);
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
