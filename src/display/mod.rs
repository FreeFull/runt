use html5ever::rcdom::{Node, NodeData};
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
    match node.data {
        NodeData::Text { ref contents } => {
            if !style.preformatted {
                let contents = &**contents.borrow();
                let contents = contents.split_whitespace().collect::<Vec<_>>().join(" ");
                print!("{} ", contents);
            } else {
                print!("{}", contents.borrow());
            }
        }
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            let mut new_style = style;
            if name.prefix == None {
                match &*name.local {
                    "b" | "strong" | "h1" | "h2" | "h3" | "h4" => {
                        new_style.bold += 1;
                    }
                    "a" => {
                        new_style.underline += 1;
                    }
                    "p" | "div" => {
                        println!("");
                    }
                    "li" => {
                        print!("\n * ");
                    }
                    "img" => {
                        let attrs = attrs.borrow();
                        let alt = attrs.iter().find(|attr| &*attr.name.local == "alt");
                        if let Some(alt) = alt {
                            print!(r#"<img alt="{}">"#, alt.value);
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
            for child in &*node.children.borrow() {
                display(child, depth + 1, new_style);
            }
            if name.prefix == None {
                match &*name.local {
                    "a" => {
                        let attrs = attrs.borrow();
                        let href = attrs.iter().find(|attr| &*attr.name.local == "href");
                        if let Some(href) = href {
                            print!(r#"<{}>"#, href.value);
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
            for child in &*node.children.borrow() {
                display(child, depth + 1, style);
            }
        }
    }
}
