use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use encoding::{Encoding, EncoderTrap};
use encoding::all::WINDOWS_1252;
use std::mem;
use grammar::*;
use self::intern::*;
use self::ruletoken::*;

mod intern;
mod ruletoken;

pub fn compile_grammar(grammar: &Grammar) -> Vec<u8> {
    let mut exported_rules = Vec::new();
    let mut imported_rules = Vec::new();

    let mut all_rules = Vec::new();
    let mut rule_name_to_id = HashMap::new();

    for (id, (name, rule)) in (1u32..).zip(&grammar.rules) {
        let rule_id = id.into();
        let name: &str = name;
        
        all_rules.push((rule_id, rule));
        rule_name_to_id.insert(name, rule_id);

        let rule_list = match *rule {
            Rule::DefinedRule(RuleVisibility::Exported, _) => exported_rules.push((id, name)),
            Rule::DefinedRule(RuleVisibility::Local, _) => (),
            Rule::ImportedRule => imported_rules.push((id, name)),
        };
    }

    let compiler = GrammarCompiler {
        exported_rules: &exported_rules,
        imported_rules: &imported_rules,
        all_rules: &all_rules,
        rule_name_to_id: &rule_name_to_id,
        words: Interner::new(),
        lists: Interner::new()
    };

    compiler.compile()
}

#[derive(Debug, Copy, Clone)]
enum ChunkType {
    Exports = 4,
    Imports = 5,
    Lists = 6,
    Words = 2,
    Rules = 3
}

struct GrammarCompiler<'b, 'a: 'b> {
    rule_name_to_id: &'b HashMap<&'a str, RuleId>,
    imported_rules: &'b [(u32, &'a str)],
    exported_rules: &'b [(u32, &'a str)],
    all_rules: &'b [(RuleId, &'a Rule)],
    words: Interner<'a, WordId>,
    lists: Interner<'a, ListId>,
}

impl<'b, 'a: 'b> GrammarCompiler<'a, 'b> {
    fn compile(mut self) -> Vec<u8> {
        let mut rule_chunk = Vec::new();
        for &(id, r) in self.all_rules.iter() {
            self.compile_rule(&mut rule_chunk, id, r);
        }
        let rule_chunk = rule_chunk;

        let words = self.words.done();
        let word_chunk = compile_id_chunk(&words);
        let lists = self.lists.done();
        let list_chunk = compile_id_chunk(&lists);

        let export_chunk = compile_id_chunk(self.exported_rules);
        let import_chunk = compile_id_chunk(self.imported_rules);
        
        let mut output = Vec::new();
        output.write_u32::<LittleEndian>(0).unwrap();
        output.write_u32::<LittleEndian>(0).unwrap();
        write_chunk(&mut output, ChunkType::Exports, export_chunk);
        write_chunk(&mut output, ChunkType::Imports, import_chunk);
        write_chunk(&mut output, ChunkType::Lists, list_chunk);
        write_chunk(&mut output, ChunkType::Words, word_chunk);
        write_chunk(&mut output, ChunkType::Rules, rule_chunk);

        output
    }

    fn compile_rule(&mut self, output: &mut Vec<u8>, id: RuleId, rule: &'a Rule) {
        match *rule {
            Rule::DefinedRule(_, ref element) => {
                let mut tokens = Vec::new();
                self.compile_element(element, &mut tokens);
                let element_data = serialize_rule_tokens(&tokens);
                write_entry(output, id.into(), element_data);
            },
            Rule::ImportedRule => ()
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

fn serialize_rule_tokens(tokens: &[RuleToken]) -> Vec<u8> {
    let mut result = Vec::new();

    for t in tokens.iter() {
        let (a, b) = t.convert();
        let probability = 0u16;
        
        result.write_u16::<LittleEndian>(a).unwrap();
        result.write_u16::<LittleEndian>(probability).unwrap();
        result.write_u32::<LittleEndian>(b).unwrap();
    }

    result
}

fn write_chunk(output: &mut Vec<u8>,
               chunk_type: ChunkType,
               mut data: Vec<u8>) {
    output.write_u32::<LittleEndian>(chunk_type as u32).unwrap();
    output.write_u32::<LittleEndian>(data.len() as u32).unwrap();
    output.append(&mut data);
}

fn write_entry(output: &mut Vec<u8>, id: u32, mut data: Vec<u8>) {
        let total_length = 2 * mem::size_of::<u32>() + data.len();

        output.write_u32::<LittleEndian>(total_length as u32).unwrap();
        output.write_u32::<LittleEndian>(id).unwrap();
        output.append(&mut data);
}

fn compile_id_chunk(entries: &[(u32, &str)]) -> Vec<u8> {
    fn add_padding(v: &mut Vec<u8>, multiple: usize) {
        let extra_padding = multiple - (v.len() % multiple);
        for _ in 0..extra_padding {
            v.push(0u8);
        }
    }
    
    let mut chunk = Vec::new();
    
    for &(id, name) in entries.iter() {
        // TODO: is this the proper encoding?
        let mut encoded = WINDOWS_1252.encode(name, EncoderTrap::Strict).unwrap();

        // make the size a multiple of 4 (and add null bytes as padding)
        add_padding(&mut encoded, 4);

        write_entry(&mut chunk, id, encoded);
    }

    chunk
}
