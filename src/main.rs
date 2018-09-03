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

use futures::prelude::*;

use bytes::Buf;

mod fetcher;
use fetcher::Fetcher;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), failure::Error> {
    let url = args().nth(1);
    let url = url.unwrap_or(String::from("https://www.rust-lang.org"));
    tokio::run(futures::lazy(move || {
        let mut fetcher = Fetcher::new().unwrap();
        fetcher
            .get(url)
            .and_then(|chunk| {
                let dom = parse_document(RcDom::default(), Default::default())
                    .from_utf8()
                    .read_from(&mut chunk.reader())?;
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
