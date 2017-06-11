mod compiler;
mod vm;
mod instructions;

use grammar::Grammar;

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CaptureName<'a> {
    Capture { name: &'a str },
    Rule { name: &'a str },
}

#[derive(Debug, Clone, Serialize)]
pub struct CaptureTree<'a, T> {
    pub name: CaptureName<'a>,
    pub slice: T,
    pub children: Vec<CaptureTree<'a, T>>,
}

pub type Match<'a> = CaptureTree<'a, (usize, usize)>;

pub struct Matcher {
    instructions: Vec<instructions::Instruction>,
}

impl Matcher {
    pub fn new(grammar: &Grammar) -> Self {
        Matcher { instructions: compiler::compile_matcher(grammar) }
    }

    pub fn perform_match<'a>(&'a self, string: &[(String, u32)]) -> Option<Match<'a>> {
        vm::perform_match(&self.instructions, string)
    }
}
