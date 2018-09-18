use super::super::types::*;
use url::Url;

#[test]
fn simple_selector() {
    let style = "div, h1, ul { background: red !important; }";
    let url = Url::parse("http://www.example.com/").unwrap();
    let stylesheet = Stylesheet::parse(style, url).unwrap();
    let mut expected = ["div", "h1", "ul"]
        .iter()
        .map(|&elem| {
            SelectorChain::Simple(SimpleSelector {
                type_selector: TypeSelector::Type(String::from(elem)),
                ..Default::default()
            })
        }).collect::<Vec<_>>();
    assert_eq!(stylesheet.rules[0].selectors, expected);
}

#[test]
fn universal_selector() {
    let style = "* { background: red !important; }";
    let url = Url::parse("http://www.example.com/").unwrap();
    let stylesheet = Stylesheet::parse(style, url).unwrap();
    let expected = SelectorChain::Simple(SimpleSelector::default());
    assert_eq!(stylesheet.rules[0].selectors[0], expected);
}
