Runt
====

Runt is a heavily work-in-progress terminal-based web browser. Currently, it is capable of
fetching a single HTML page from a HTTP or HTTPS URL, or the local file system.

## How to run

Note: Due to the usage of the 2018 edition of Rust, Runt currently requires the nightly compiler.
Once Rust 2018 ends up in stable, the code should still work without any changes.

```sh
# Fetch and display the default page (currently https://www.rust-lang.org/)
cargo run

# Fetch and display a given URL
cargo run -- 'https://www.google.com/'

# Open a local HTML file for display
cargo run -- '/path/to/file.html'
cargo run -- 'file:///path/to/file.html'
```

## Inspirations

* [Browsh](https://www.brow.sh/)
* [Lynx](http://lynx.invisible-island.net/)
* [Links 2](http://links.twibright.com/)
* [Let's build a browser engine](https://limpet.net/mbrubeck/2014/08/08/toy-layout-engine-1.html)