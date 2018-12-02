use cssparser::{
    parse_important, AtRuleParser, BasicParseError, BasicParseErrorKind, CowRcStr,
    DeclarationListParser, DeclarationParser, Delimiter, ParseError, Parser, ParserInput,
    QualifiedRuleParser, RuleListParser, SourceLocation, ToCss, Token,
};
use std::marker::PhantomData;
use url::Url;

use super::*;

#[derive(Debug)]
pub enum Error {
    Url(url::ParseError),
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::Url(err)
    }
}

#[derive(Debug)]
enum ParseErrorKind<'i> {
    Selector(selectors::parser::SelectorParseErrorKind<'i>),
    Other,
}

impl<'i> From<selectors::parser::SelectorParseErrorKind<'i>> for ParseErrorKind<'i> {
    fn from(err: selectors::parser::SelectorParseErrorKind<'i>) -> ParseErrorKind<'i> {
        ParseErrorKind::Selector(err)
    }
}

impl Stylesheet {
    pub fn parse(stylesheet: &str, url: &Url) -> Result<Stylesheet, Error> {
        let mut input = ParserInput::new(stylesheet);
        let mut parser = Parser::new(&mut input);
        let rules = RuleListParser::new_for_stylesheet(&mut parser, TopLevelRuleParser::new())
            .filter_map(|item| item.ok())
            .collect();
        Ok(Stylesheet {
            rules,
            base_url: url.join("")?,
        })
    }
}

struct TopLevelRuleParser<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a> TopLevelRuleParser<'a> {
    fn new() -> TopLevelRuleParser<'a> {
        TopLevelRuleParser {
            _phantom: PhantomData,
        }
    }
}

impl<'i> QualifiedRuleParser<'i> for TopLevelRuleParser<'i> {
    type Prelude = selectors::SelectorList<super::super::selectors::Impl>;
    type QualifiedRule = Rule;
    type Error = ParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let selector_parser = crate::css::selectors::Parser;
        selectors::SelectorList::parse(&selector_parser, input).map_err(|e| e.into())
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _location: SourceLocation,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let declaration_parser = DeclarationListParser::new(input, RuleDeclarationParser {});
        Ok(Rule {
            selectors: prelude,
            declarations: declaration_parser.filter_map(|dec| dec.ok()).collect(),
        })
    }
}

impl<'i> AtRuleParser<'i> for TopLevelRuleParser<'i> {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = Rule;
    type Error = ParseErrorKind<'i>;
}

struct RuleDeclarationParser {}

impl<'i> DeclarationParser<'i> for RuleDeclarationParser {
    type Declaration = Declaration;
    type Error = ParseErrorKind<'i>;
    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Declaration, ParseError<'i, Self::Error>> {
        let mut values = vec![];
        input.parse_until_before(Delimiter::Bang, |input| {
            loop {
                match input.next() {
                    Ok(token) => {
                        values.push(Value(token.to_css_string()));
                    }
                    Err(BasicParseError {
                        kind: BasicParseErrorKind::EndOfInput,
                        ..
                    }) => break,
                    err => {
                        err?;
                    }
                }
            }
            Ok(())
        })?;
        let important = input.r#try(parse_important).is_ok();
        input.expect_exhausted()?;
        Ok(Declaration {
            name: String::from(&*name),
            value: values,
            important: important,
        })
    }
}

impl<'i> AtRuleParser<'i> for RuleDeclarationParser {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = Declaration;
    type Error = ParseErrorKind<'i>;
}
