#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grammar {
    pub rules: Box<[Rule]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub exported: bool,
    pub element: Element,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub definition: Option<RuleDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Element {
    Sequence { children: Box<[Element]> },
    Alternative { children: Box<[Element]> },
    Repetition { child: Box<Element> },
    Optional { child: Box<Element> },
    Capture { key: String, child: Box<Element> },
    Word { text: String },
    RuleRef { name: String },
    List { name: String },
}
