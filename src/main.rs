#[macro_use]
extern crate failure;
extern crate bytes;
extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio;

use std::default::Default;
use std::env::args;

use html5ever::parse_document;
use html5ever::rcdom::{Node, NodeData, RcDom};
use html5ever::tendril::TendrilSink;

use futures::stream::Stream;
use futures::Future;

use bytes::Buf;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), failure::Error> {
    let url = args().nth(1);
    let url = url
        .as_ref()
        .map(|url| &url[..])
        .unwrap_or("https://www.rust-lang.org")
        .parse()?;
    tokio::run(futures::lazy(|| {
        let https = hyper_tls::HttpsConnector::new(4).unwrap();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        client
            .get(url)
            .from_err()
            .and_then(|response| {
                if response.status() != hyper::StatusCode::OK {
                    bail!("HTTP status code: {}", response.status())
                } else {
                    Ok(response.into_body().concat2().from_err())
                }
            }).and_then(|body| body)
            .and_then(|body| {
                let dom = parse_document(RcDom::default(), Default::default())
                    .from_utf8()
                    .read_from(&mut body.reader())?;
                walk(&dom.document, 0);
                Ok(())
            }).then(|result| {
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
                Ok(())
            })
    }));
    Ok(())
}

fn walk(node: &Node, depth: u32) {
    for _ in 0..depth {
        print!(" ");
    }
    match node.data {
        NodeData::Comment { ref contents } => {
            println!("Comment: {}", contents);
        }
        NodeData::Document => {
            println!("Document");
        }
        NodeData::Doctype {
            ref name,
            ref public_id,
            ref system_id,
        } => {
            println!("Doctype: {} {} {}", name, public_id, system_id);
        }
        NodeData::Text { ref contents } => {
            println!("Text: {}", contents.borrow());
        }
        NodeData::Element {
            ref name,
            ref attrs,
            ref template_contents,
            ref mathml_annotation_xml_integration_point,
        } => {
            println!(
                "Element: {:?} {:?} {}",
                name,
                attrs.borrow(),
                mathml_annotation_xml_integration_point
            );
        }
        NodeData::ProcessingInstruction {
            ref target,
            ref contents,
        } => {
            println!("ProcessingInstruction: {} {}", target, contents);
        }
    }
    for child in &*node.children.borrow() {
        walk(child, depth + 1);
    }
}
