#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grammar {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub exported: bool,
    pub definition: Element,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Element {
    Sequence { children: Vec<Element> },
    Alternative { children: Vec<Element> },
    Repetition { child: Box<Element> },
    Optional { child: Box<Element> },
    Capture { name: String, child: Box<Element> },
    Word { text: String },
    RuleRef { name: String },
    List { name: String },
    Dictation,
    DictationWord,
    SpellingLetter,
}
