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
enum RuleToken {
    Nested(NestedPosition, NestedType),
    Basic(BasicType, u32)
}

impl RuleToken {
    fn word(id: u32) -> Self {
        RuleToken::Basic(BasicType::Word, id)
    }

    fn rule(id: u32) -> Self {
        RuleToken::Basic(BasicType::Rule, id)
    }

    fn list(id: u32) -> Self {
        RuleToken::Basic(BasicType::List, id)
    }
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
enum Rule {
    ExportedRule(Element),
    LocalRule(Element),
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

struct Void {
}

struct Interner<T> {
    name_to_id: HashMap<String, u32>,
    names: Vec<String>,
    values: Vec<T>
}

impl<T> Interner<T> {
    fn new() -> Self {
        Interner {
            name_to_id: HashMap::new(),
            names: Vec::new(),
            values: Vec::new()
        }
    }
    
    fn intern(&mut self, s: String, v: T) -> u32 {
        if let Some(&id) = self.name_to_id.get(&s) {
            id
        } else {
            let id = (self.names.len() + 1) as u32;
            self.name_to_id.insert(s.clone(), id);
            self.names.push(s);
            self.values.push(v);
            id
        }
    }

    fn done(self) -> (HashMap<String, u32>, Vec<String>, Vec<T>) {
        (self.name_to_id, self.names, self.values)
    }
}

struct GrammarCompiler {
    buffer: Vec<u8>,
    rule_name_to_id: HashMap<String, u32>,
    words: Interner<Void>,
    lists: Interner<Void>
}

impl GrammarCompiler {
    fn new(rule_name_to_id: HashMap<String, u32>,
           words: Interner<Void>,
           lists: Interner<Void>) -> Self {
        GrammarCompiler {
            buffer: Vec::new(),
            rule_name_to_id: rule_name_to_id,
            words: words,
            lists: lists
        }
    }

    fn compile_element(&mut self, element: &Element, output: &mut Vec<RuleToken>) {
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
                let id = self.words.intern(word.clone(), Void {});
                output.push(RuleToken::word(id));
            }
            Element::Rule(ref name) => {
                // TODO: handle missing rule
                let id = self.rule_name_to_id.get(name).unwrap();
                output.push(RuleToken::rule(*id));
            }
            Element::List(ref name) => {
                let id = self.lists.intern(name.clone(), Void {});
                output.push(RuleToken::list(id));
            }
        }
    }
}

fn compile_grammar(mut grammar: Grammar) -> Vec<u8> {
    let mut exported_rules = Vec::new();
    let mut imported_rules = Vec::new();

    let mut rule_interner = Interner::new();

    for (name, rule) in grammar.rules.drain() {
        let rule_list = match rule {
            Rule::ExportedRule(_) => Some(&mut exported_rules),
            Rule::LocalRule(_) => None,
            Rule::ImportedRule => Some(&mut imported_rules)
        };
        
        let id = rule_interner.intern(name.clone(), rule);

        if let Some(rules) = rule_list {
            rules.push((id, name));
        }
    }

    let (rule_name_to_id, _, all_rules) = rule_interner.done();
    
    Vec::new()
}

fn compile_id_chunk<T: Write>(output: &mut T,
                              chunk_type: ChunkType,
                              entries: Vec<(u32, String)>) {
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
