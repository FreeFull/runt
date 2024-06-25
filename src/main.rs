use std::env::args;
use std::path::Path;

use anyhow::format_err;
use url::Url;

mod display;
mod fetcher;
mod page;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let url = args().nth(1);
    let url = url.unwrap_or(String::from("https://www.rust-lang.org/"));
    let parsed_url;
    if url.starts_with("file:") || url.starts_with("http:") || url.starts_with("https:") {
        parsed_url = Url::parse(&url)
    } else {
        // Try to interpret the argument as a file path
        let path = Path::new(&url).canonicalize()?;
        parsed_url = Ok(Url::from_file_path(path)
            .map_err(|()| format_err!("Failed to convert path to URL: {}", url))?);
    }
    let page = page::fetch(parsed_url?).await?;
    display::display(&page.document)?;
    Ok(())
}
