mod compiler;
mod vm;
mod captures;
mod instructions;

pub use self::captures::{CaptureTree, Match};
use engine::WordInfo;
use grammar::Grammar;

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
