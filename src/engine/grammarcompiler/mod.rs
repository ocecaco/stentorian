use byteorder::{LittleEndian, WriteBytesExt};
use encoding::{Encoding, EncoderTrap};
use encoding::all::WINDOWS_1252;
use std::mem;
use grammar::*;
use self::rulecompiler::*;

mod intern;
mod ruletoken;

mod rulecompiler {
    use super::ruletoken::*;
    use super::intern::*;
    use grammar::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;

    type IdNamePairs<'a> = Vec<(u32, &'a str)>;

    pub struct CompilerResult<'a> {
        pub imported_rules: IdNamePairs<'a>,
        pub exported_rules: IdNamePairs<'a>,
        pub rule_name_to_id: HashMap<&'a str, RuleId>,
        pub words: IdNamePairs<'a>,
        pub lists: IdNamePairs<'a>,
    }

    pub struct RuleCompiler<'a> {
        rule_counter: RuleId,
        imported_rules: IdNamePairs<'a>,
        exported_rules: IdNamePairs<'a>,
        rule_name_to_id: HashMap<&'a str, RuleId>,
        words: Interner<'a>,
        lists: Interner<'a>,
    }

    impl<'a> RuleCompiler<'a> {
        pub fn new() -> Self {
            RuleCompiler {
                rule_counter: 0,
                imported_rules: Vec::new(),
                exported_rules: Vec::new(),
                rule_name_to_id: HashMap::new(),
                words: Interner::new(),
                lists: Interner::new(),
            }
        }

        pub fn done(self) -> CompilerResult<'a> {
            CompilerResult {
                imported_rules: self.imported_rules,
                exported_rules: self.exported_rules,
                rule_name_to_id: self.rule_name_to_id,
                words: self.words.done(),
                lists: self.lists.done(),
            }
        }

        pub fn compile_rule(&mut self, rule: &'a Rule) -> (RuleId, Vec<u8>) {
            let mut tokens = Vec::new();
            self.compile_element(&rule.definition, &mut tokens);
            let result = serialize_rule_tokens(&tokens);

            let id = self.declare_rule(&rule.name);
            if rule.exported {
                self.exported_rules.push((id, &rule.name));
            }

            (id, result)
        }

        fn declare_rule(&mut self, name: &'a str) -> RuleId {
            match self.rule_name_to_id.entry(name) {
                Entry::Occupied(_) => panic!("duplicate rule name"),
                Entry::Vacant(entry) => {
                    self.rule_counter += 1;
                    let id = self.rule_counter;

                    entry.insert(id);
                    id
                },
            }
        }

        fn add_imported_rule(&mut self, name: &'static str) -> RuleId {
            match self.rule_name_to_id.entry(name) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => {
                    self.rule_counter += 1;
                    let id = self.rule_counter;

                    entry.insert(id);
                    self.imported_rules.push((id, name));
                    id
                },
            }
        }

        fn compile_element(&mut self, element: &'a Element, output: &mut Vec<RuleToken>) {
            match *element {
                Element::Sequence { ref children } => {
                    output.push(SEQUENCE_START);
                    for c in children.iter() {
                        self.compile_element(c, output);
                    }
                    output.push(SEQUENCE_END);
                }
                Element::Alternative { ref children } => {
                    output.push(ALTERNATIVE_START);
                    for c in children.iter() {
                        self.compile_element(c, output);
                    }
                    output.push(ALTERNATIVE_END);
                }
                Element::Repetition { ref child } => {
                    output.push(REPETITION_START);
                    self.compile_element(child, output);
                    output.push(REPETITION_END);
                }
                Element::Optional { ref child } => {
                    output.push(OPTIONAL_START);
                    self.compile_element(child, output);
                    output.push(OPTIONAL_END);
                }
                Element::Word { ref text } => {
                    let id = self.words.intern(text);
                    output.push(RuleToken::Word(id));
                }
                Element::RuleRef { ref name } => {
                    // TODO: handle missing rule
                    #[allow(get_unwrap)]
                    let id = self.rule_name_to_id.get::<str>(name).unwrap();
                    output.push(RuleToken::Rule(*id));
                }
                Element::List { ref name } => {
                    let id = self.lists.intern(name);
                    output.push(RuleToken::List(id));
                }
                Element::Capture { ref child, .. } => {
                    self.compile_element(child, output);
                }
                Element::Dictation => {
                    let id = self.add_imported_rule("dgndictation");
                    output.push(RuleToken::Rule(id));
                }
                Element::DictationWord => {
                    let id = self.add_imported_rule("dgnwords");
                    output.push(RuleToken::Rule(id));
                }
                Element::SpellingLetter => {
                    let id = self.add_imported_rule("dgnletters");
                    output.push(RuleToken::Rule(id));
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
}

pub fn compile_grammar(grammar: &Grammar) -> Vec<u8> {
    let mut compiler = RuleCompiler::new();

    let mut rule_chunk = Vec::new();
    for r in grammar.rules.iter() {
        let (id, compiled) = compiler.compile_rule(r);
        write_entry(&mut rule_chunk, id, compiled);
    }
    let rule_chunk = rule_chunk;

    let compile_result = compiler.done();

    let word_chunk = compile_id_chunk(&compile_result.words);
    let list_chunk = compile_id_chunk(&compile_result.lists);

    let export_chunk = compile_id_chunk(&compile_result.exported_rules);
    let import_chunk = compile_id_chunk(&compile_result.imported_rules);

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

#[derive(Debug, Copy, Clone)]
enum ChunkType {
    Exports = 4,
    Imports = 5,
    Lists = 6,
    Words = 2,
    Rules = 3,
}

fn write_chunk(output: &mut Vec<u8>, chunk_type: ChunkType, mut data: Vec<u8>) {
    output
        .write_u32::<LittleEndian>(chunk_type as u32)
        .unwrap();
    output
        .write_u32::<LittleEndian>(data.len() as u32)
        .unwrap();
    output.append(&mut data);
}

fn write_entry(output: &mut Vec<u8>, id: u32, mut data: Vec<u8>) {
    let total_length = 2 * mem::size_of::<u32>() + data.len();

    output
        .write_u32::<LittleEndian>(total_length as u32)
        .unwrap();
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
