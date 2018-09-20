use std::default::Default;
use std::env::args;
use std::path::Path;

use failure::format_err;
use url::Url;

mod css;
mod display;
mod fetcher;
mod page;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), failure::Error> {
    let url = args().nth(1);
    let url = url.unwrap_or(String::from("https://www.rust-lang.org/en-US/"));
    let parsed_url;
    if url.starts_with("file:") || url.starts_with("http:") || url.starts_with("https:") {
        parsed_url = Url::parse(&url)
    } else {
        // Try to interpret the argument as a file path
        let path = Path::new(&url).canonicalize()?;
        parsed_url = Ok(Url::from_file_path(path)
            .map_err(|()| format_err!("Failed to convert path to URL: {}", url))?);
    }
    let page = page::fetch(parsed_url?)?;
    display::display(&page.dom.document, 0, Default::default());
    println!("");
    Ok(())
}
