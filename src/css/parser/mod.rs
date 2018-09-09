use cssparser::{
    AtRuleParser, ParseError, Parser, ParserInput, QualifiedRuleParser, RuleListParser,
    SourceLocation,
};
use failure::Error;
use std::marker::PhantomData;
use url::Url;

use super::types::*;

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
        Ok(vec![])
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        location: SourceLocation,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        Ok(Rule {
            selectors: prelude,
            declarations: vec![],
        })
    }
}

impl<'a> AtRuleParser<'a> for TopLevelRuleParser<'a> {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = Rule;
    type Error = ();
}
