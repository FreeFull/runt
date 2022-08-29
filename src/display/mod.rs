use kuchiki::{Node, NodeData};
use termion::style;

#[derive(Copy, Clone, Debug, Default)]
pub struct Style {
    bold: u32,
    italic: u32,
    underline: u32,
    preformatted: bool,
}

impl Style {
    fn show(&self) {
        print!("{}", style::Reset);
        match self.bold {
            0 => {}
            _ => print!("{}", style::Bold),
        }
        match self.italic {
            0 => {}
            _ => print!("{}", style::Italic),
        }
        match self.underline {
            0 => {}
            _ => print!("{}", style::Underline),
        }
    }
}

pub fn display(node: &Node, depth: u32, style: Style) {
    style.show();
    match node.data() {
        NodeData::Text(contents) => {
            if !style.preformatted {
                let contents = &**contents.borrow();
                let contents = contents.split_whitespace().collect::<Vec<_>>().join(" ");
                print!("{} ", contents);
            } else {
                print!("{}", contents.borrow());
            }
        }
        NodeData::Element(ref data) => {
            let mut new_style = style;
            if data.name.prefix == None {
                match &*data.name.local {
                    "b" | "strong" | "h1" | "h2" | "h3" | "h4" => {
                        new_style.bold += 1;
                    }
                    "i" | "em" => {
                        new_style.italic += 1;
                    }
                    "a" | "u" => {
                        new_style.underline += 1;
                    }
                    "p" | "div" => {
                        println!("");
                    }
                    "li" => {
                        print!("\n * ");
                    }
                    "img" => {
                        let attrs = data.attributes.borrow();
                        let alt = attrs.get("alt");
                        if let Some(alt) = alt {
                            print!(r#"<img alt="{}">"#, alt);
                        } else {
                            print!("<img>");
                        }
                        return;
                    }
                    "pre" | "textarea" => {
                        new_style.preformatted = true;
                    }
                    "script" | "head" | "style" => {
                        return;
                    }
                    "q" => {
                        print!("\"");
                    }
                    _ => {}
                }
            }
            {
                let mut node = node.first_child();
                while let Some(child) = node {
                    display(&child, depth + 1, new_style);
                    node = child.next_sibling();
                }
            }
            if data.name.prefix == None {
                match &*data.name.local {
                    "a" => {
                        let attrs = data.attributes.borrow();
                        let href = attrs.get("href");
                        if let Some(href) = href {
                            print!(r#"<{}>"#, href);
                        } else {
                            print!("<>");
                        }
                    }
                    "q" => {
                        print!("\"");
                    }
                    _ => {}
                }
            }
            style.show()
        }
        _ => {
            let mut node = node.first_child();
            while let Some(child) = node {
                display(&child, depth + 1, style);
                node = child.next_sibling();
            }
        }
    }
}
