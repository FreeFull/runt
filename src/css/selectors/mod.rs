#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Impl;
impl selectors::SelectorImpl for Impl {
    type ExtraMatchingData = ();
    type AttrValue = String;
    type Identifier = String;
    type ClassName = String;
    type LocalName = String;
    type NamespaceUrl = String;
    type NamespacePrefix = String;
    type BorrowedNamespaceUrl = str;
    type BorrowedLocalName = str;
    type NonTSPseudoClass = super::stylesheet::PseudoClass;
    type PseudoElement = super::stylesheet::PseudoElement;
}

pub struct Parser;

impl<'i> selectors::Parser<'i> for Parser {
    type Impl = Impl;
    type Error = selectors::parser::SelectorParseErrorKind<'i>;
}
