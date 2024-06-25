use std::{io, time::Duration};

use kuchiki::{Node, NodeData};
use ratatui::{crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
}, style::Styled};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

#[derive(Copy, Clone, Debug, Default)]
struct State {
    should_quit: bool,
    offset_y: u16,
}

pub fn display(node: &Node) -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut page = Paragraph::new(parse(node)).wrap(Wrap { trim: true });

    let mut state = State::default();
    while !state.should_quit {
        terminal.draw(|frame| ui(frame, &page))?;
        handle_events(&mut state)?;
        page = page.scroll((state.offset_y, 0));
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn ui(frame: &mut Frame, page: &Paragraph) {
    frame.render_widget(page, frame.size());
}
fn handle_events(state: &mut State) -> io::Result<()> {
    if event::poll(Duration::from_secs(1))? {
        match event::read()? {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: _,
                state: _,
            }) => {
                if !modifiers.is_empty() {
                    return Ok(());
                }
                if code == KeyCode::Char('q') {
                    state.should_quit = true;
                    return Ok(());
                }
                if code == KeyCode::Up {
                    state.offset_y = state.offset_y.saturating_sub(1);
                }
                if code == KeyCode::Down {
                    state.offset_y = state.offset_y.saturating_add(1);
                }
                if code == KeyCode::PageUp {
                    state.offset_y = state.offset_y.saturating_sub(10);
                }
                if code == KeyCode::PageDown {
                    state.offset_y = state.offset_y.saturating_add(10);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse(node: &Node) -> Text<'static> {
    fn parse_inner(node: &Node, text: &mut Text<'static>, mut style: Style) {
        match node.data() {
            NodeData::Text(contents) => {
                text.push_span(contents.borrow().clone().set_style(style));
            }
            NodeData::Element(ref data) => {
                if data.name.prefix == None {
                    match &*data.name.local {
                        "b" | "strong" | "h1" | "h2" | "h3" | "h4" => {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        "i" | "em" => {
                            style = style.add_modifier(Modifier::ITALIC);
                        }
                        "a" | "u" => {
                            style = style.add_modifier(Modifier::UNDERLINED);
                        }
                        "p" | "div" => {
                            text.push_line("");
                        }
                        "li" => {
                            text.push_line(" * ");
                        }
                        "img" => {
                            let attrs = data.attributes.borrow();
                            let alt = attrs.get("alt");
                            if let Some(alt) = alt {
                                text.push_line(format!(r#"<img alt="{}">"#, alt));
                            } else {
                                text.push_line("<img>");
                            }
                            return;
                        }
                        "pre" | "textarea" => {}
                        "script" | "head" | "style" => {
                            return;
                        }
                        _ => {}
                    }
                }
                {
                    let mut node = node.first_child();
                    while let Some(child) = node {
                        parse_inner(&child, text, style);
                        node = child.next_sibling();
                    }
                }
                if data.name.prefix == None {
                    match &*data.name.local {
                        "a" => {
                            let attrs = data.attributes.borrow();
                            let href = attrs.get("href");
                            if let Some(href) = href {
                                text.push_span(format!(r#"<{}>"#, href));
                            } else {
                                text.push_span("<>");
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                let mut node = node.first_child();
                while let Some(child) = node {
                    parse_inner(&child, text, style);
                    node = child.next_sibling();
                }
            }
        }
    }

    let mut text = Text::default();
    parse_inner(node, &mut text, Style::new());
    text
}
