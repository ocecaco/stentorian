use std::collections::HashMap;
use std::collections::hash_map::Entry;
use super::instructions::*;

pub fn perform_match<'a, 'b: 'c, 'c>(program: &'a [Instruction],
                                     string: &'c [&'b str])
                                     -> MatchResult<'a> {
    let vm = Vm::new(program, string);
    vm.perform_match()
}

#[derive(Debug, Copy, Clone)]
pub enum Capture {
    Started(usize),
    Complete(usize, usize),
}

#[derive(Debug, Clone)]
struct Thread<'a, 'b: 'c, 'c> {
    instructions: &'a [Instruction],
    string: &'c [&'b str],
    program_pointer: usize,
    string_pointer: usize,
    call_stack: Vec<usize>,
    captures: HashMap<&'a str, Capture>,
    top_level_rule: Option<u32>,
    progress: HashMap<usize, usize>,
}

pub type MatchResult<'a> = Option<(u32, HashMap<&'a str, Capture>)>;

impl<'a, 'b: 'c, 'c> Thread<'a, 'b, 'c> {
    fn new(instructions: &'a [Instruction], string: &'c [&'b str]) -> Self {
        Thread {
            instructions: instructions,
            string: string,
            program_pointer: 0,
            string_pointer: 0,
            call_stack: Vec::new(),
            captures: HashMap::new(),
            top_level_rule: None,
            progress: HashMap::new(),
        }
    }

    fn run(mut self, threads: &mut Vec<Thread<'a, 'b, 'c>>) -> MatchResult<'a> {
        loop {
            let next = &self.instructions[self.program_pointer];
            self.program_pointer += 1;

            match *next {
                Instruction::TopLevelRule(r) => {
                    if self.top_level_rule.is_none() {
                        self.top_level_rule = Some(r);
                    }
                }
                Instruction::Literal(ref word) => {
                    if self.string_pointer >= self.string.len() {
                        return None;
                    }

                    let current_word = self.string[self.string_pointer];

                    if current_word == word {
                        self.string_pointer += 1;
                    } else {
                        return None;
                    }
                }
                Instruction::Match => {
                    if let Some(return_address) = self.call_stack.pop() {
                        self.program_pointer = return_address;
                    } else if self.string_pointer == self.string.len() {
                        return Some((self.top_level_rule.unwrap(), self.captures));
                    } else {
                        return None;
                    }
                }
                Instruction::CaptureStart(ref name) => {
                    self.captures
                        .insert(name, Capture::Started(self.string_pointer));
                }
                Instruction::CaptureStop(ref name) => {
                    #[allow(get_unwrap)]
                    let c = *self.captures.get::<str>(name)
                        .expect(&format!("capture {} stopped without being started", name));
                    if let Capture::Started(start) = c {
                        self.captures
                            .insert(name, Capture::Complete(start, self.string_pointer));
                    }
                }
                Instruction::RuleCall(JumpTarget::Concrete(address)) => {
                    self.call_stack.push(self.program_pointer);
                    self.program_pointer = address;
                }
                Instruction::RuleCall(JumpTarget::Symbolic(_)) |
                Instruction::Jump(JumpTarget::Symbolic(_)) => {
                    panic!("found symbolic jump instruction");
                }
                Instruction::Jump(JumpTarget::Concrete(address)) => {
                    self.program_pointer = address;
                }
                Instruction::Split(ref targets) => {
                    let (first, rest) = targets.split_first().unwrap();

                    for t in rest.iter().rev() {
                        if let JumpTarget::Concrete(address) = *t {
                            let mut branch = self.clone();
                            branch.program_pointer = address;
                            threads.push(branch);
                        } else {
                            panic!("found symbolic jump instruction");
                        }
                    }

                    if let JumpTarget::Concrete(address) = *first {
                        self.program_pointer = address;
                    } else {
                        panic!("found symbolic jump instruction");
                    }
                }
                Instruction::List(ref name) => {
                    unimplemented!();
                }
                Instruction::Progress => {
                    // make sure we've progressed since the last time
                    // we were here to avoid infinite loop
                    let pc = self.program_pointer;
                    let current = self.string_pointer;

                    let entry = self.progress.entry(pc);
                    match entry {
                        Entry::Occupied(mut e) => {
                            let previous = e.get_mut();

                            // stop we haven't made progress
                            if current == *previous {
                                return None;
                            }

                            *previous = current;
                        }
                        Entry::Vacant(e) => {
                            e.insert(current);
                        }
                    }
                }
                Instruction::NoOp |
                Instruction::Label(_) => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Vm<'a, 'b: 'c, 'c> {
    program: &'a [Instruction],
    string: &'c [&'b str],
    threads: Vec<Thread<'a, 'b, 'c>>,
}

impl<'a, 'b: 'c, 'c> Vm<'a, 'b, 'c> {
    fn new(program: &'a [Instruction], string: &'c [&'b str]) -> Self {
        Vm {
            program: program,
            string: string,
            threads: Vec::new(),
        }
    }

    fn perform_match(mut self) -> MatchResult<'a> {
        self.threads.push(Thread::new(self.program, self.string));

        while let Some(t) = self.threads.pop() {
            let result = t.run(&mut self.threads);

            if result.is_some() {
                return result;
            }
        }

        None
    }
}
