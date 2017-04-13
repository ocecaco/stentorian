#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: Box<[Rule]>,
}

#[derive(Debug, Copy, Clone)]
pub enum RuleVisibility {
    Exported,
    Local,
}

#[derive(Debug, Clone)]
pub enum RuleDefinition {
    Defined(RuleVisibility, Element),
    Imported,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub definition: RuleDefinition,
}

#[derive(Debug, Clone)]
pub enum Element {
    Sequence(Box<[Element]>),
    Alternative(Box<[Element]>),
    Repetition(Box<Element>),
    Optional(Box<Element>),
    Capture(String, Box<Element>),
    Literal(String),
    RuleRef(String),
    List(String),
}
