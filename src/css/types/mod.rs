use std;
use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
    pub base_url: Url,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<SelectorChain>,
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectorChain {
    Simple(SimpleSelector),
    Descendant(Box<SelectorChain>, SimpleSelector),
    Child(Box<SelectorChain>, SimpleSelector),
    AdjacentSibling(Box<SelectorChain>, SimpleSelector),
}

impl SelectorChain {
    pub fn specificity(&self) -> Specificity {
        match *self {
            SelectorChain::Simple(ref sel) => sel.specificity(),
            SelectorChain::Descendant(ref chain, ref selector) => {
                chain.specificity() + selector.specificity()
            }
            SelectorChain::Child(ref chain, ref selector) => {
                chain.specificity() + selector.specificity()
            }
            SelectorChain::AdjacentSibling(ref chain, ref selector) => {
                chain.specificity() + selector.specificity()
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimpleSelector {
    pub type_selector: TypeSelector,
    pub attribute_selectors: Vec<AttributeSelector>,
    pub id_selectors: Vec<IdSelector>,
    pub class_selectors: Vec<ClassSelector>,
    pub pseudo_elements: Vec<PseudoElement>,
    pub pseudo_classes: Vec<PseudoClass>,
}

impl SimpleSelector {
    pub fn specificity(&self) -> Specificity {
        let mut specificity = Specificity { a: 0, b: 0, c: 0 };
        specificity.a = self.id_selectors.iter().count() as u32;
        specificity.b = (self.class_selectors.iter().count()
            + self.attribute_selectors.iter().count()
            + self.pseudo_classes.iter().count()) as u32;
        match self.type_selector {
            TypeSelector::Type(_) => specificity.c = 1,
            _ => {}
        }
        specificity.c += self.pseudo_elements.iter().count() as u32;
        specificity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeSelector {
    Type(String),
    Universal,
}

impl Default for TypeSelector {
    fn default() -> Self {
        TypeSelector::Universal
    }
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
    FirstLine,
    FirstLetter,
    Before,
    After,
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
