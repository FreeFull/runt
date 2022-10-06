use bytes::Bytes;
use kuchiki::{traits::*, ElementData, Node, NodeData, NodeRef};
use std::collections::HashMap;
use std::future::Future;
use url::Url;

use crate::fetcher::{self, Data, Error, Fetcher};

pub struct Page {
    pub url: Url,
    pub resources: Resources,
    pub document: kuchiki::NodeRef,
}

pub async fn fetch(url: Url) -> Result<Page, anyhow::Error> {
    let mut fetcher = fetcher::Fetcher::new()?;
    let page = fetcher.get_with_redirect(url.clone(), 30).await?;
    let document = kuchiki::parse_html().one(&*String::from_utf8_lossy(page.as_ref()));
    let responses = Box::into_pin(fetch_resources_from_dom(
        &url,
        document.clone(),
        &mut fetcher,
        FetchState::default(),
    ))
    .await;
    let mut resources = Resources::default();
    for response in responses {
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
                url: url.clone(),
                parts,
                data,
            },
        );
    }
    Ok(Page {
        url,
        resources,
        document,
    })
}

fn fetch_resources_from_dom<'a>(
    origin: &'a Url,
    node: NodeRef,
    fetcher: &'a mut Fetcher,
    mut fetch_state: FetchState,
) -> Box<dyn Future<Output = Vec<Result<Data, Error>>> + 'a> {
    Box::new(async move {
        let mut responses = Vec::new();
        match node.data() {
            NodeData::Element(ElementData {
                name,
                attributes: attrs,
                ..
            }) => match &*name.local.to_ascii_lowercase() {
                "head" => {
                    fetch_state = FetchState::InsideHead;
                }
                "body" => {
                    fetch_state = FetchState::InsideBody;
                }
                "link" => {
                    let attrs = attrs.borrow();
                    if attrs.get("rel") == Some("stylesheet") {
                        if let Some(url) = attrs.get("href") {
                            if let Ok(url) = origin.join(url) {
                                responses.push(fetcher.get_with_redirect(url, 30).await);
                            }
                        }
                    }
                }
                "img" => {
                    let attrs = attrs.borrow();
                    if let Some(url) = attrs.get("src") {
                        if let Ok(url) = origin.join(&url) {
                            responses.push(fetcher.get_with_redirect(url, 30).await);
                        }
                    }
                }
                "template" => {
                    return responses;
                }
                _ => {}
            },
            _ => {}
        }
        {
            for child in node.children() {
                responses.append(
                    &mut Box::into_pin(fetch_resources_from_dom(
                        origin,
                        child,
                        fetcher,
                        fetch_state,
                    ))
                    .await,
                );
            }
        }
        responses
    })
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
