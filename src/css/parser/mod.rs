use cssparser::{
    parse_important, AtRuleParser, BasicParseError, BasicParseErrorKind, CowRcStr,
    DeclarationListParser, DeclarationParser, Delimiter, ParseError, Parser, ParserInput,
    QualifiedRuleParser, RuleListParser, SourceLocation, ToCss, Token,
};
use failure::Error;
use std::marker::PhantomData;
use url::Url;

use super::types::*;

#[cfg(test)]
mod tests;

impl Stylesheet {
    fn parse(stylesheet: &str, url: Url) -> Result<Stylesheet, Error> {
        let mut input = ParserInput::new(stylesheet);
        let mut parser = Parser::new(&mut input);
        let rules = RuleListParser::new_for_stylesheet(&mut parser, TopLevelRuleParser::new())
            .collect::<Result<_, _>>()
            .unwrap();
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
    fn new() -> TopLevelRuleParser<'static> {
        TopLevelRuleParser {
            _phantom: PhantomData,
        }
    }
}

impl<'i> QualifiedRuleParser<'i> for TopLevelRuleParser<'i> {
    type Prelude = Vec<SelectorChain>;
    type QualifiedRule = Rule;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        input.parse_comma_separated(parse_selector_chain)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        location: SourceLocation,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let declaration_parser = DeclarationListParser::new(input, RuleDeclarationParser {});
        Ok(Rule {
            selectors: prelude,
            declarations: declaration_parser
                .collect::<Result<_, _>>()
                .map_err(|x| x.0)?,
        })
    }
}

impl<'a> AtRuleParser<'a> for TopLevelRuleParser<'a> {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = Rule;
    type Error = ();
}

struct RuleDeclarationParser {}

impl<'i> DeclarationParser<'i> for RuleDeclarationParser {
    type Declaration = Declaration;
    type Error = ();
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
        let important = input.try(parse_important).is_ok();
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
    type Error = ();
}

fn parse_selector_chain<'i, 'tt>(
    parser: &mut Parser<'i, 'tt>,
) -> Result<SelectorChain, ParseError<'i, ()>> {
    Ok(SelectorChain::Simple(parse_simple_selector(parser)?))
}

fn parse_simple_selector<'i, 'tt>(
    parser: &mut Parser<'i, 'tt>,
) -> Result<SimpleSelector, ParseError<'i, ()>> {
    let mut selector = SimpleSelector::default();
    match parser.next() {
        Ok(&Token::Ident(ref ident)) => {
            selector.type_selector = TypeSelector::Type(String::from(&**ident));
        }
        Ok(_) => unimplemented!(),
        Err(err) => return Err(ParseError::from(err)),
    }
    Ok(selector)
}
