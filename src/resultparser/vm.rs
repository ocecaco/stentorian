use std::collections::HashMap;
use std::collections::hash_map::Entry;
use super::instructions::*;

pub fn perform_match<'a, 'c>(program: &'a [Instruction],
                                     string: &'c [(String, u32)])
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
struct Thread<'a, 'c> {
    instructions: &'a [Instruction],
    string: &'c [(String, u32)],
    program_pointer: usize,
    string_pointer: usize,
    rule_stack: Vec<u32>,
    call_stack: Vec<usize>,
    captures: HashMap<&'a str, Capture>,
    progress: HashMap<usize, usize>,
}

pub type MatchResult<'a> = Option<(u32, HashMap<&'a str, Capture>)>;

impl<'a, 'c> Thread<'a, 'c> {
    fn new(instructions: &'a [Instruction], string: &'c [(String, u32)]) -> Self {
        Thread {
            instructions: instructions,
            string: string,
            program_pointer: 0,
            string_pointer: 0,
            rule_stack: Vec::new(),
            call_stack: Vec::new(),
            captures: HashMap::new(),
            progress: HashMap::new(),
        }
    }

    fn run(mut self, threads: &mut Vec<Thread<'a, 'c>>) -> MatchResult<'a> {
        loop {
            let next = &self.instructions[self.program_pointer];
            self.program_pointer += 1;

            match *next {
                Instruction::RuleEntry(id) => {
                    self.rule_stack.push(id);
                }
                Instruction::Literal(ref word) => {
                    if self.string_pointer >= self.string.len() {
                        return None;
                    }

                    let (ref current_word, _id) = self.string[self.string_pointer];

                    if current_word == word {
                        self.string_pointer += 1;
                    } else {
                        return None;
                    }
                }
                Instruction::Match => {
                    let current_rule = self.rule_stack.pop().unwrap();
                    if let Some(return_address) = self.call_stack.pop() {
                        self.program_pointer = return_address;
                    } else if self.string_pointer == self.string.len() {
                        return Some((current_rule, self.captures));
                    } else {
                        return None;
                    }
                }
                Instruction::CaptureStart(ref name) => {
                    self.captures
                        .insert(name, Capture::Started(self.string_pointer));
                }
                Instruction::CaptureStop(ref name) => {
                    let c = *self.captures.get::<str>(name)
                        .expect(&format!("capture {} stopped without being started", name));
                    if let Capture::Started(start) = c {
                        self.captures
                            .insert(name, Capture::Complete(start, self.string_pointer));
                    }
                }
                Instruction::RuleCall(ref t) => {
                    self.call_stack.push(self.program_pointer);
                    self.program_pointer = t.address();
                }
                Instruction::Jump(ref t) => {
                    self.program_pointer = t.address();
                }
                Instruction::Split(ref targets) => {
                    let (first, rest) = targets.split_first().unwrap();

                    for t in rest.iter().rev() {
                        let mut branch = self.clone();
                        branch.program_pointer = t.address();
                        threads.push(branch);
                    }

                    self.program_pointer = first.address();
                }
                Instruction::List(ref _name) => {
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
struct Vm<'a, 'c> {
    program: &'a [Instruction],
    string: &'c [(String, u32)],
    threads: Vec<Thread<'a, 'c>>,
}

impl<'a, 'c> Vm<'a, 'c> {
    fn new(program: &'a [Instruction], string: &'c [(String, u32)]) -> Self {
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
