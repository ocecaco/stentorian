use std::collections::HashMap;
use super::instructions::*;

struct Vm<'a> {
    rule_programs: Vec<Option<&'a [Instruction]>>,
    threads: HashMap<usize, Thread<'a>>
}

struct Thread<'a> {
    instructions: &'a [Instruction],
    program_pointer: usize
}
