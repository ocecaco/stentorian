mod compiler;
mod vm;
mod instructions;

use std::collections::HashMap;
use grammar::Grammar;

#[derive(Debug, Clone)]
pub struct Match<'a> {
    pub top_level_rule: u32,
    pub captures: HashMap<&'a str, (usize, usize)>,
}

pub struct Matcher {
    instructions: Vec<instructions::Instruction>,
}

impl Matcher {
    pub fn new(grammar: &Grammar) -> Self {
        Matcher {
            instructions: compiler::compile_matcher(grammar),
        }
    }

    pub fn perform_match<'a>(&'a self, string: &[(String, u32)]) -> Option<Match<'a>> {
        vm::perform_match(&self.instructions, string)
    }
}
