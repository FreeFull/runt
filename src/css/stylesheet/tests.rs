use super::*;
use url::Url;

use selectors::parser::{Component, LocalName, Selector, SelectorList};

#[test]
fn simple_selector() {
    let style = "div, h1, ul { background: red !important; }";
    let url = Url::parse("http://www.example.com/").unwrap();
    let stylesheet = Stylesheet::parse(style, &url).unwrap();
    let expected = ["div", "h1", "ul"]
        .iter()
        .cloned()
        .map(|selector| {
            Component::LocalName(LocalName {
                name: String::from(selector),
                lower_name: selector.to_lowercase(),
            })
        })
        .map(|component| Selector::from_vec(vec![component], 1))
        .collect();
    let expected = selectors::SelectorList::from_vec(expected);
    assert_eq!(stylesheet.rules[0].selectors, expected);
}

#[test]
fn universal_selector() {
    let style = "* { background: red !important; }";
    let url = Url::parse("http://www.example.com/").unwrap();
    let stylesheet = Stylesheet::parse(style, &url).unwrap();
    let expected = SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::ExplicitUniversalType],
        0,
    )]);
    assert_eq!(stylesheet.rules[0].selectors, expected);
}
