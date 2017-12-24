mod compiler;
mod vm;
mod captures;
mod instructions;

use grammar::Grammar;

pub use self::captures::{CaptureTree, Match};

pub struct Matcher {
    instructions: Vec<instructions::Instruction>,
}

impl Matcher {
    pub fn new(grammar: &Grammar) -> Self {
        Matcher {
            instructions: compiler::compile_matcher(grammar),
        }
    }

    pub fn perform_match<'a>(&'a self, string: &[(String, u32)]) -> Option<Vec<Match<'a>>> {
        vm::perform_match(&self.instructions, string)
    }
}
