extern crate failure;
extern crate html5ever;
extern crate reqwest;

use std::default::Default;

use html5ever::parse_document;
use html5ever::rcdom::{Node, NodeData, RcDom};
use html5ever::tendril::TendrilSink;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), failure::Error> {
    let body = reqwest::get("https://www.rust-lang.org")?.text()?;
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut body.as_bytes())?;
    walk(&dom.document, 0);
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
