#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: Box<[(String, Rule)]>,
}

#[derive(Debug, Copy, Clone)]
pub enum RuleVisibility {
    Exported,
    Local,
}

#[derive(Debug, Clone)]
pub enum Rule {
    DefinedRule(RuleVisibility, Element),
    ImportedRule,
}

#[derive(Debug, Clone)]
pub enum Element {
    Sequence(Box<[Element]>),
    Alternative(Box<[Element]>),
    Repetition(Box<Element>),
    Optional(Box<Element>),
    Capture(String, Box<Element>),
    Literal(String),
    Rule(String),
    List(String),
}
