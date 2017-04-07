use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: BTreeMap<String, Rule>
}

#[derive(Debug, Clone)]
pub enum RuleVisibility {
    Exported,
    Local
}

#[derive(Debug, Clone)]
pub enum Rule {
    DefinedRule(RuleVisibility, Element),
    ImportedRule
}

#[derive(Debug, Clone)]
pub enum Element {
    Sequence(Box<[Element]>),
    Alternative(Box<[Element]>),
    Repetition(Box<Element>),
    Optional(Box<Element>),
    Literal(String),
    Rule(String),
    List(String)
}
