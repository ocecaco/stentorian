use byteorder::{LittleEndian, WriteBytesExt};
use std::mem;
use grammar::*;
use self::errors::*;
use self::ruletoken::*;
use self::intern::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
pub use self::ruletoken::RuleId;

mod intern;
mod ruletoken;

pub mod errors {
    error_chain! {
        errors {
            UnknownRule(name: String) {
                description("unknown rule name in grammar definition")
                    display("unknown rule name in grammar definition: {}", name)
            }

            DuplicateRule(name: String) {
                description("duplicate rule name in grammar definition")
                    display("duplicate rule name in grammar definition: {}", name)
            }

            ReservedRule(name: String) {
                description("reserved rule name in grammar definition")
                    display("reserved rule name in grammar definition: {}", name)
            }
        }
    }
}

pub fn compile_command_grammar(grammar: &Grammar) -> Result<Vec<u8>> {
    let compiler = GrammarCompiler::new(grammar);
    compiler.compile_grammar()
}

pub fn compile_select_grammar(select_words: &[String], through_words: &[String]) -> Vec<u8> {
    let mut output = Vec::new();

    output.write_u32::<LittleEndian>(10).unwrap();
    output.write_u32::<LittleEndian>(1).unwrap();

    let select_chunk = compile_id_chunk(select_words.iter().map(|s| (0, s as &str)));
    write_chunk(&mut output, ChunkType::SelectWords, select_chunk);
    let through_chunk = compile_id_chunk(through_words.iter().map(|s| (0, s as &str)));
    write_chunk(&mut output, ChunkType::ThroughWords, through_chunk);

    output
}

pub fn compile_dictation_grammar() -> Vec<u8> {
    let mut output = Vec::new();

    output.write_u32::<LittleEndian>(2).unwrap();
    output.write_u32::<LittleEndian>(1).unwrap();

    output
}

pub enum ImportedRule {
    Dictation,
    DictationWord,
    SpellingLetter,
}

impl ImportedRule {
    fn name(&self) -> &'static str {
        match *self {
            ImportedRule::Dictation => "dgndictation",
            ImportedRule::DictationWord => "dgnwords",
            ImportedRule::SpellingLetter => "dgnletters",
        }
    }

    pub fn offset(&self) -> u32 {
        match *self {
            ImportedRule::Dictation => 1,
            ImportedRule::DictationWord => 2,
            ImportedRule::SpellingLetter => 3,
        }
    }
}

type IdNamePairs<'a> = Vec<(u32, &'a str)>;

struct GrammarCompiler<'a> {
    imported_rules: IdNamePairs<'a>,
    exported_rules: IdNamePairs<'a>,
    rule_name_to_id: HashMap<&'a str, RuleId>,
    words: Interner<'a>,
    lists: Interner<'a>,
    grammar: &'a Grammar,
}

impl<'a> GrammarCompiler<'a> {
    fn new(grammar: &'a Grammar) -> Self {
        GrammarCompiler {
            imported_rules: Vec::new(),
            exported_rules: Vec::new(),
            rule_name_to_id: HashMap::new(),
            words: Interner::new(),
            lists: Interner::new(),
            grammar: grammar,
        }
    }

    fn compile_grammar(mut self) -> Result<Vec<u8>> {
        let mut rule_chunk = Vec::new();
        for (id, r) in (1u32..).zip(self.grammar.rules.iter()) {
            let compiled = self.compile_rule(id, r)?;
            write_entry(&mut rule_chunk, id, compiled);
        }
        let rule_chunk = rule_chunk;

        let words = self.words.done();
        let word_chunk = compile_id_chunk(words);
        let lists = self.lists.done();
        let list_chunk = compile_id_chunk(lists);

        let export_chunk = compile_id_chunk(self.exported_rules);
        let import_chunk = compile_id_chunk(self.imported_rules);

        let mut output = Vec::new();
        output.write_u32::<LittleEndian>(0).unwrap();
        output.write_u32::<LittleEndian>(1).unwrap();
        write_chunk(&mut output, ChunkType::Exports, export_chunk);
        write_chunk(&mut output, ChunkType::Imports, import_chunk);
        write_chunk(&mut output, ChunkType::Lists, list_chunk);
        write_chunk(&mut output, ChunkType::Words, word_chunk);
        write_chunk(&mut output, ChunkType::Rules, rule_chunk);

        Ok(output)
    }

    fn compile_rule(&mut self, id: RuleId, rule: &'a Rule) -> Result<Vec<u8>> {
        let mut tokens = Vec::new();
        self.compile_element(&rule.definition, &mut tokens)?;
        let result = serialize_rule_tokens(&tokens);

        self.declare_rule(id, &rule.name)?;
        if rule.exported {
            self.exported_rules.push((id, &rule.name));
        }

        Ok(result)
    }

    fn declare_rule(&mut self, id: RuleId, name: &'a str) -> Result<()> {
        match self.rule_name_to_id.entry(name) {
            Entry::Occupied(_) => Err(ErrorKind::DuplicateRule(name.to_string()).into()),
            Entry::Vacant(entry) => {
                entry.insert(id);
                Ok(())
            }
        }
    }

    fn add_imported_rule(&mut self, rule: ImportedRule) -> RuleId {
        match self.rule_name_to_id.entry(rule.name()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = self.grammar.rules.len() as u32 + rule.offset();

                entry.insert(id);
                self.imported_rules.push((id, rule.name()));
                id
            }
        }
    }

    fn compile_element(&mut self, element: &'a Element, output: &mut Vec<RuleToken>) -> Result<()> {
        match *element {
            Element::Sequence { ref children } => {
                output.push(SEQUENCE_START);
                for c in children.iter() {
                    self.compile_element(c, output)?;
                }
                output.push(SEQUENCE_END);
            }
            Element::Alternative { ref children } => {
                output.push(ALTERNATIVE_START);
                for c in children.iter() {
                    self.compile_element(c, output)?;
                }
                output.push(ALTERNATIVE_END);
            }
            Element::Repetition { ref child } => {
                output.push(REPETITION_START);
                self.compile_element(child, output)?;
                output.push(REPETITION_END);
            }
            Element::Optional { ref child } => {
                output.push(OPTIONAL_START);
                self.compile_element(child, output)?;
                output.push(OPTIONAL_END);
            }
            Element::Word { ref text } => {
                let id = self.words.intern(text);
                output.push(RuleToken::Word(id));
            }
            Element::RuleRef { ref name } => {
                let maybe_id = self.rule_name_to_id.get::<str>(name);
                let result = maybe_id.ok_or_else(|| ErrorKind::UnknownRule(name.clone()))?;
                output.push(RuleToken::Rule(*result));
            }
            Element::List { ref name } => {
                let id = self.lists.intern(name);
                output.push(RuleToken::List(id));
            }
            Element::Capture { ref child, .. } => {
                self.compile_element(child, output)?;
            }
            Element::Dictation => {
                let id = self.add_imported_rule(ImportedRule::Dictation);
                output.push(RuleToken::Rule(id));
            }
            Element::DictationWord => {
                let id = self.add_imported_rule(ImportedRule::DictationWord);
                output.push(RuleToken::Rule(id));
            }
            Element::SpellingLetter => {
                let id = self.add_imported_rule(ImportedRule::SpellingLetter);
                output.push(RuleToken::Rule(id));
            }
        };

        Ok(())
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

#[derive(Debug, Copy, Clone)]
enum ChunkType {
    Exports = 4,
    Imports = 5,
    Lists = 6,
    Words = 2,
    Rules = 3,
    SelectWords = 0x1017,
    ThroughWords = 0x1018,
}

fn write_chunk(output: &mut Vec<u8>, chunk_type: ChunkType, mut data: Vec<u8>) {
    output.write_u32::<LittleEndian>(chunk_type as u32).unwrap();
    output.write_u32::<LittleEndian>(data.len() as u32).unwrap();
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

fn compile_id_chunk<'a, E>(entries: E) -> Vec<u8>
where
    E: IntoIterator<Item = (u32, &'a str)>,
{
    fn add_padding(v: &mut Vec<u8>, multiple: usize) {
        let extra_padding = multiple - (v.len() % multiple);
        for _ in 0..extra_padding {
            v.push(0u8);
        }
    }

    fn encode(s: &str) -> Vec<u8> {
        let mut result = Vec::new();
        for c in s.encode_utf16() {
            result.write_u16::<LittleEndian>(c).unwrap();
        }
        result
    }

    let mut chunk = Vec::new();

    for (id, name) in entries {
        let mut encoded = encode(name);

        // make sure word is terminated by at least *two* null bytes
        // after padding
        encoded.push(0u8);

        // make the size a multiple of 4 (and add null bytes as padding)
        add_padding(&mut encoded, 4);

        write_entry(&mut chunk, id, encoded);
    }

    chunk
}
