mod captures;
mod compiler;
mod instructions;
mod vm;

pub use self::captures::{CaptureTree, Match};
use crate::engine::WordInfo;
use crate::grammar::Grammar;

pub struct Matcher {
    instructions: Vec<instructions::Instruction>,
}

impl Matcher {
    pub fn new(grammar: &Grammar) -> Self {
        Matcher {
            instructions: compiler::compile_matcher(grammar),
        }
    }

    pub fn perform_match<'a>(&'a self, string: &[WordInfo]) -> Option<Vec<Match<'a>>> {
        vm::perform_match(&self.instructions, string)
    }
}
