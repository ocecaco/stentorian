use std::io::Write;
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use encoding::{Encoding, EncoderTrap};
use encoding::all::WINDOWS_1252;
use std::mem;

#[derive(Debug, Copy, Clone)]
enum ChunkType {
    Exports = 4,
    Imports = 5,
    Lists = 6,
    Words = 2
}

#[derive(Debug, Copy, Clone)]
enum NestedPosition {
    Start = 1,
    End = 2
}

#[derive(Debug, Copy, Clone)]
enum NestedType {
    Sequence = 1,
    Alternative = 2,
    Repetition = 3,
    Optional = 4
}

#[derive(Debug, Copy, Clone)]
enum BasicType {
    Word = 3,
    Rule = 4,
    List = 6
}

#[derive(Debug, Copy, Clone)]
struct WordId(u32);

#[derive(Debug, Copy, Clone)]
struct RuleId(u32);

#[derive(Debug, Copy, Clone)]
struct ListId(u32);

impl From<u32> for WordId {
    fn from(v: u32) -> Self {
        WordId(v)
    }
}

impl From<u32> for RuleId {
    fn from(v: u32) -> Self {
        RuleId(v)
    }
}

impl From<u32> for ListId {
    fn from(v: u32) -> Self {
        ListId(v)
    }
}

#[derive(Debug, Copy, Clone)]
enum RuleToken {
    Nested(NestedPosition, NestedType),
    Word(WordId),
    Rule(RuleId),
    List(ListId)
}

const SEQUENCE_START: RuleToken = RuleToken::Nested(NestedPosition::Start, NestedType::Sequence);
const SEQUENCE_END: RuleToken = RuleToken::Nested(NestedPosition::End, NestedType::Sequence);

const ALTERNATIVE_START: RuleToken = RuleToken::Nested(NestedPosition::Start, NestedType::Alternative);
const ALTERNATIVE_END: RuleToken = RuleToken::Nested(NestedPosition::End, NestedType::Alternative);

const REPETITION_START: RuleToken = RuleToken::Nested(NestedPosition::Start, NestedType::Repetition);
const REPETITION_END: RuleToken = RuleToken::Nested(NestedPosition::End, NestedType::Repetition);

const OPTIONAL_START: RuleToken = RuleToken::Nested(NestedPosition::Start, NestedType::Optional);
const OPTIONAL_END: RuleToken = RuleToken::Nested(NestedPosition::End, NestedType::Optional);

#[derive(Debug, Clone)]
struct Grammar {
    rules: HashMap<String, Rule>
}

#[derive(Debug, Clone)]
enum RuleVisibility {
    Exported,
    Local
}

#[derive(Debug, Clone)]
enum Rule {
    DefinedRule(RuleVisibility, Element),
    ImportedRule
}

#[derive(Debug, Clone)]
enum Element {
    Sequence(Box<[Element]>),
    Alternative(Box<[Element]>),
    Repetition(Box<Element>),
    Optional(Box<Element>),
    Literal(String),
    Rule(String),
    List(String)
}

struct Interner<'a, U> {
    name_to_id: HashMap<&'a str, U>,
    names: Vec<&'a str>,
}

impl<'a, U: From<u32> + Copy> Interner<'a, U> {
    fn new() -> Self {
        Interner {
            name_to_id: HashMap::new(),
            names: Vec::new(),
        }
    }
    
    fn intern(&mut self, s: &'a str) -> U {
        if let Some(&id) = self.name_to_id.get(s) {
            id
        } else {
            self.names.push(s);

            let id = U::from(self.names.len() as u32);
            self.name_to_id.insert(s, id);
            id
        }
    }

    fn done(self) -> (HashMap<&'a str, U>, Vec<&'a str>) {
        (self.name_to_id, self.names)
    }
}

fn serialize_rule_tokens(tokens: &[RuleToken]) -> Vec<u8> {
    fn handle_token(token: &RuleToken) -> (u16, u32) {
        match *token {
            RuleToken::Nested(pos, ty) => {
                (pos as u16, ty as u32)
            },
            RuleToken::Word(word_id) => {
                (BasicType::Word as u16, word_id.0)
            }
            RuleToken::Rule(rule_id) => {
                (BasicType::Rule as u16, rule_id.0)
            }
            RuleToken::List(list_id) => {
                (BasicType::List as u16, list_id.0)
            }
        }
    }
    
    let mut result = Vec::new();

    for t in tokens.iter() {
        let (a, b) = handle_token(t);
        let probability = 0u16;
        
        result.write_u16::<LittleEndian>(a).unwrap();
        result.write_u16::<LittleEndian>(probability).unwrap();
        result.write_u32::<LittleEndian>(b).unwrap();
    }

    result
}


struct GrammarCompiler<'a> {
    grammar: &'a Grammar,
    buffer: Vec<u8>,
    rule_name_to_id: HashMap<&'a str, RuleId>,
    words: Interner<'a, WordId>,
    lists: Interner<'a, ListId>
}

impl<'a> GrammarCompiler<'a> {
    fn new(grammar: &'a Grammar,
           rule_name_to_id: HashMap<&'a str, RuleId>) -> Self {
        GrammarCompiler {
            grammar: grammar,
            buffer: Vec::new(),
            rule_name_to_id: rule_name_to_id,
            words: Interner::new(),
            lists: Interner::new()
        }
    }

    fn compile(&mut self) -> Vec<u8> {
        Vec::new()
    }


    fn compile_rule(&mut self, rule: &'a Rule) -> Option<Vec<RuleToken>> {
        match *rule {
            Rule::DefinedRule(_, ref element) => {
                let mut tokens = Vec::new();
                self.compile_element(element, &mut tokens);
                Some(tokens)
            },
            Rule::ImportedRule => None
        }
    }

    fn compile_element(&mut self, element: &'a Element, output: &mut Vec<RuleToken>) {
        match *element {
            Element::Sequence(ref children) => {
                output.push(SEQUENCE_START);
                for c in children.iter() {
                    self.compile_element(c, output);
                }
                output.push(SEQUENCE_END);
            },
            Element::Alternative(ref children) => {
                output.push(ALTERNATIVE_START);
                for c in children.iter() {
                    self.compile_element(c, output);
                }
                output.push(ALTERNATIVE_END);
            },
            Element::Repetition(ref child) => {
                output.push(REPETITION_START);
                self.compile_element(child, output);
                output.push(REPETITION_END);
            },
            Element::Optional(ref child) => {
                output.push(OPTIONAL_START);
                self.compile_element(child, output);
                output.push(OPTIONAL_END);
            },
            Element::Literal(ref word) => {
                let id = self.words.intern(word);
                output.push(RuleToken::Word(id));
            }
            Element::Rule(ref name) => {
                // TODO: handle missing rule
                let id = self.rule_name_to_id.get::<str>(name).unwrap();
                output.push(RuleToken::Rule(*id));
            }
            Element::List(ref name) => {
                let id = self.lists.intern(name);
                output.push(RuleToken::List(id));
            }
        }
    }
}

fn compile_grammar(grammar: &Grammar) -> Vec<u8> {
    let mut exported_rules = Vec::new();
    let mut imported_rules = Vec::new();

    let mut all_rules = Vec::new();
    let mut rule_name_to_id = HashMap::new();

    for (name, rule) in &grammar.rules {
        all_rules.push(rule);
        let id = all_rules.len();
        rule_name_to_id.insert(name, id);

        let rule_list = match *rule {
            Rule::DefinedRule(RuleVisibility::Exported, _) => exported_rules.push((id, name)),
            Rule::DefinedRule(RuleVisibility::Local, _) => (),
            Rule::ImportedRule => imported_rules.push((id, name)),
        };
    }

    Vec::new()
}

fn compile_id_chunk<T: Write>(output: &mut T,
                              chunk_type: ChunkType,
                              entries: Vec<(u32, &str)>) {
    let mut chunk = Vec::new();
    
    for (id, name) in entries {
        // TODO: is this the proper encoding?
        let mut encoded = WINDOWS_1252.encode(&name, EncoderTrap::Strict).unwrap();

        // make the size a multiple of 4 (and add null bytes as padding)
        let extra_padding = 4 - (encoded.len() % 4);
        for _ in 0..extra_padding {
            encoded.push(0u8);
        }

        let encoded = encoded;

        let total_length = 2 * mem::size_of::<u32>() + encoded.len();

        chunk.write_u32::<LittleEndian>(total_length as u32).unwrap();
        chunk.write_u32::<LittleEndian>(id).unwrap();

        chunk.write_all(&encoded).unwrap();
    }

    let chunk = chunk;

    output.write_u32::<LittleEndian>(chunk_type as u32).unwrap();
    output.write_u32::<LittleEndian>(chunk.len() as u32).unwrap();
    output.write_all(&chunk).unwrap();
}
