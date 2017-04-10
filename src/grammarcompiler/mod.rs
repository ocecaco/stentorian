use byteorder::{LittleEndian, WriteBytesExt};
use encoding::{Encoding, EncoderTrap};
use encoding::all::WINDOWS_1252;
use std::mem;
use grammar::*;
use self::rulecompiler::*;
use self::ruletoken::*;

mod intern;
mod ruletoken;

mod rulecompiler {
    use super::ruletoken::*;
    use super::intern::*;
    use grammar::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::collections::HashMap;

    type IdNamePairs<'a> = Vec<(u32, &'a str)>;

    pub struct RuleCompiler<'a> {
        rule_name_to_id: HashMap<&'a str, RuleId>,
        words: Interner<'a, WordId>,
        lists: Interner<'a, ListId>,
    }

    impl<'a> RuleCompiler<'a> {
        pub fn new() -> Self {
            RuleCompiler {
                rule_name_to_id: HashMap::new(),
                words: Interner::new(),
                lists: Interner::new()
            }
        }
        
        pub fn done(self) -> (IdNamePairs<'a>, IdNamePairs<'a>) {
            (self.words.done(), self.lists.done())
        }
        
        pub fn compile_rule(&mut self, id: RuleId, name: &'a str, rule: &'a Rule) -> Option<Vec<u8>> {
            // TODO: check for duplicate rule name
            self.rule_name_to_id.insert(name, id);
            
            match *rule {
                Rule::DefinedRule(_, ref element) => {
                    let mut tokens = Vec::new();
                    self.compile_element(element, &mut tokens);
                    let element_data = serialize_rule_tokens(&tokens);
                    Some(element_data)
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
                    #[allow(get_unwrap)]
                    let id = self.rule_name_to_id.get::<str>(name).unwrap();
                    output.push(RuleToken::Rule(*id));
                }
                Element::List(ref name) => {
                    let id = self.lists.intern(name);
                    output.push(RuleToken::List(id));
                }
                Element::Capture(_, ref child) => {
                    self.compile_element(child, output);
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

struct PreprocessResult<'a> {
    exported_rules: Vec<(u32, &'a str)>,
    imported_rules: Vec<(u32, &'a str)>,
    all_rules: Vec<(RuleId, &'a str, &'a Rule)>,
}

fn preprocess(grammar: &Grammar) -> PreprocessResult {
    let mut exported_rules = Vec::new();
    let mut imported_rules = Vec::new();

    let mut all_rules = Vec::new();

    for (id, &(ref name, ref rule)) in (1u32..).zip(grammar.rules.iter()) {
        let rule_id = id.into();
        let name: &str = name;
        
        all_rules.push((rule_id, name, rule));

        match *rule {
            Rule::DefinedRule(RuleVisibility::Exported, _) => exported_rules.push((id, name)),
            Rule::DefinedRule(RuleVisibility::Local, _) => (),
            Rule::ImportedRule => imported_rules.push((id, name)),
        }
    }

    PreprocessResult {
        exported_rules: exported_rules,
        imported_rules: imported_rules,
        all_rules: all_rules,
    }
}


pub fn compile_grammar(grammar: &Grammar) -> Vec<u8> {
    let preprocess_result = preprocess(grammar);

    let mut compiler = RuleCompiler::new();

    let mut rule_chunk = Vec::new();
    for &(id, name, r) in &preprocess_result.all_rules {
        if let Some(compiled_rule) = compiler.compile_rule(id, name, r) {
            write_entry(&mut rule_chunk, id.into(), compiled_rule);
        }
    }
    let rule_chunk = rule_chunk;

    let (words, lists) = compiler.done();

    let word_chunk = compile_id_chunk(&words);
    let list_chunk = compile_id_chunk(&lists);

    let export_chunk = compile_id_chunk(&preprocess_result.exported_rules);
    let import_chunk = compile_id_chunk(&preprocess_result.imported_rules);
    
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
    Rules = 3
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
