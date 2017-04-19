pub type WordId = u32;
pub type RuleId = u32;
pub type ListId = u32;

#[derive(Debug, Copy, Clone)]
pub enum NestedPosition {
    Start = 1,
    End = 2,
}

#[derive(Debug, Copy, Clone)]
pub enum NestedType {
    Sequence = 1,
    Alternative = 2,
    Repetition = 3,
    Optional = 4,
}

#[derive(Debug, Copy, Clone)]
pub enum BasicType {
    Word = 3,
    Rule = 4,
    List = 6,
}

#[derive(Debug, Copy, Clone)]
pub enum RuleToken {
    Nested(NestedPosition, NestedType),
    Word(WordId),
    Rule(RuleId),
    List(ListId),
}

impl RuleToken {
    pub fn convert(&self) -> (u16, u32) {
        match *self {
            RuleToken::Nested(pos, ty) => (pos as u16, ty as u32),
            RuleToken::Word(word_id) => (BasicType::Word as u16, word_id),
            RuleToken::Rule(rule_id) => (BasicType::Rule as u16, rule_id),
            RuleToken::List(list_id) => (BasicType::List as u16, list_id),
        }
    }
}

pub const SEQUENCE_START: RuleToken = RuleToken::Nested(NestedPosition::Start,
                                                        NestedType::Sequence);
pub const SEQUENCE_END: RuleToken = RuleToken::Nested(NestedPosition::End, NestedType::Sequence);

pub const ALTERNATIVE_START: RuleToken = RuleToken::Nested(NestedPosition::Start,
                                                           NestedType::Alternative);
pub const ALTERNATIVE_END: RuleToken = RuleToken::Nested(NestedPosition::End,
                                                         NestedType::Alternative);

pub const REPETITION_START: RuleToken = RuleToken::Nested(NestedPosition::Start,
                                                          NestedType::Repetition);
pub const REPETITION_END: RuleToken = RuleToken::Nested(NestedPosition::End,
                                                        NestedType::Repetition);

pub const OPTIONAL_START: RuleToken = RuleToken::Nested(NestedPosition::Start,
                                                        NestedType::Optional);
pub const OPTIONAL_END: RuleToken = RuleToken::Nested(NestedPosition::End, NestedType::Optional);
