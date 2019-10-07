use super::captures::{CaptureBuilder, Match};
use super::instructions::Instruction;
use crate::engine::WordInfo;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub fn perform_match<'a, 'c>(
    program: &'a [Instruction],
    string: &'c [WordInfo],
) -> Option<Vec<Match<'a>>> {
    let mut threads = Vec::new();
    threads.push(Thread::new(program, string));

    while let Some(t) = threads.pop() {
        let result = t.run(&mut threads).ok();

        if result.is_some() {
            return result;
        }
    }

    None
}

type Result<T> = ::std::result::Result<T, ()>;

#[derive(Debug, Clone)]
struct Thread<'a, 'c> {
    instructions: &'a [Instruction],
    string: &'c [WordInfo],
    program_pointer: usize,
    string_pointer: usize,
    call_stack: Vec<usize>,
    captures: CaptureBuilder<'a>,
    progress: HashMap<usize, usize>,
}

impl<'a, 'c> Thread<'a, 'c> {
    fn new(instructions: &'a [Instruction], string: &'c [WordInfo]) -> Self {
        Thread {
            instructions: instructions,
            string: string,
            program_pointer: 0,
            string_pointer: 0,
            call_stack: Vec::new(),
            captures: CaptureBuilder::new(),
            progress: HashMap::new(),
        }
    }

    fn match_token(&mut self, word: Option<&'a str>) -> Result<()> {
        let current = self.string.get(self.string_pointer);
        if let Some(word_info) = current {
            if let Some(word) = word {
                if word != word_info.text {
                    return Err(());
                }
            }

            self.string_pointer += 1;
            return Ok(());
        } else {
            return Err(());
        }
    }

    fn run(mut self, threads: &mut Vec<Thread<'a, 'c>>) -> Result<Vec<Match<'a>>> {
        loop {
            let next = &self.instructions[self.program_pointer];
            self.program_pointer += 1;

            match *next {
                Instruction::Literal(ref grammar_word) => {
                    self.match_token(Some(grammar_word))?;
                }
                Instruction::AnyWord => {
                    self.match_token(None)?;
                }
                Instruction::CaptureStart(ref name) => {
                    self.captures.capture_start(name, self.string_pointer);
                }
                Instruction::CaptureStop => {
                    self.captures.capture_stop(self.string_pointer);
                }
                Instruction::Return => {
                    if let Some(return_address) = self.call_stack.pop() {
                        self.program_pointer = return_address;
                    } else if self.string_pointer == self.string.len() {
                        return Ok(self.captures.done());
                    } else {
                        return Err(());
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
                                return Err(());
                            }

                            *previous = current;
                        }
                        Entry::Vacant(e) => {
                            e.insert(current);
                        }
                    }
                }
                Instruction::NoOp | Instruction::Label(_) => {}
            }
        }
    }
}
