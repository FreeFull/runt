use url::Url;

mod parser;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
    pub base_url: Url,
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub selectors: selectors::SelectorList<super::selectors::Impl>,
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeSelector {
    Has { attribute: String },
    Equals { attribute: String, value: String },
    OneOf { attribute: String, value: String },
    Subcode { attribute: String, value: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct IdSelector(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct ClassSelector(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PseudoElement {
    After,
    Backdrop,
    Before,
    Cue,
    FirstLetter,
    FirstLine,
    GrammarError,
    Marker,
    Placeholder,
    Selection,
    SpellingError,
}

impl cssparser::ToCss for PseudoElement {
    fn to_css<W>(&self, writer: &mut W) -> Result<(), std::fmt::Error>
    where
        W: std::fmt::Write,
    {
        writer.write_str("::")?;
        writer.write_str(match *self {
            PseudoElement::After => "after",
            PseudoElement::Backdrop => "backdrop",
            PseudoElement::Before => "before",
            PseudoElement::Cue => "cue",
            PseudoElement::FirstLetter => "first-letter",
            PseudoElement::FirstLine => "first-line",
            PseudoElement::GrammarError => "grammar-error",
            PseudoElement::Marker => "marker",
            PseudoElement::Placeholder => "placeholder",
            PseudoElement::Selection => "selection",
            PseudoElement::SpellingError => "spelling-error",
        })
    }
}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = super::selectors::Impl;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PseudoClass {
    FirstChild,
    Link,
    Visited,
    Hover,
    Active,
    Focus,
    Lang(String),
}

impl cssparser::ToCss for PseudoClass {
    fn to_css<W>(&self, writer: &mut W) -> Result<(), std::fmt::Error>
    where
        W: std::fmt::Write,
    {
        use self::PseudoClass::*;
        write!(
            writer,
            "{}",
            match *self {
                FirstChild => ":first-child",
                Link => ":link",
                Visited => ":visited",
                Hover => ":hover",
                Active => ":active",
                Focus => ":focus",
                Lang(ref lang) => {
                    return write!(writer, ":lang({})", lang);
                }
            }
        )
    }
}

impl selectors::parser::NonTSPseudoClass for PseudoClass {
    type Impl = super::selectors::Impl;
    fn is_active_or_hover(&self) -> bool {
        self == &PseudoClass::Active || self == &PseudoClass::Hover
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Vec<Value>,
    pub important: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Value(pub String);

/// https://www.w3.org/TR/selectors/#specificity
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Specificity {
    pub a: u32,
    pub b: u32,
    pub c: u32,
}

impl std::ops::Add for Specificity {
    type Output = Specificity;

    fn add(self, rhs: Specificity) -> Specificity {
        Specificity {
            a: self.a + rhs.a,
            b: self.b + rhs.b,
            c: self.c + rhs.c,
        }
    }
}
