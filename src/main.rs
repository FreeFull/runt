#[macro_use]
extern crate failure;
extern crate bytes;
extern crate cssparser;
extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate termion;
extern crate tokio;
extern crate url;

use std::default::Default;
use std::env::args;

use html5ever::parse_document;
use html5ever::rcdom::RcDom;
use html5ever::tendril::TendrilSink;

use futures::prelude::*;

use bytes::Buf;

mod fetcher;
use fetcher::Fetcher;

mod display;

mod css;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), failure::Error> {
    let url = args().nth(1);
    let url = url.unwrap_or(String::from("https://www.rust-lang.org/en-US/"));
    tokio::run(futures::lazy(move || {
        let mut fetcher = Fetcher::new().unwrap();
        fetcher
            .get(url)
            .and_then(|chunk| {
                let dom = parse_document(RcDom::default(), Default::default())
                    .from_utf8()
                    .read_from(&mut chunk.reader())?;
                display::display(&dom.document, 0, Default::default());
                println!("");
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
