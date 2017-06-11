use std::collections::HashMap;
use std::collections::hash_map::Entry;
use super::{Match, CaptureTree, CaptureName};
use super::instructions::*;

pub fn perform_match<'a, 'c>(instructions: &'a [Instruction],
                             string: &'c [(String, u32)])
                             -> Option<Match<'a>> {
    let vm = Vm::new(instructions, string);
    vm.perform_match()
}

#[derive(Debug, Copy, Clone)]
enum Capture {
    Started(usize),
    Complete(usize, usize),
}

impl Capture {
    fn complete(&self) -> (usize, usize) {
        if let Capture::Complete(a, b) = *self {
            (a, b)
        } else {
            panic!("attempt to unwrap incomplete capture");
        }
    }
}

fn complete_capture_tree<'a>(tree: &CaptureTree<'a, Capture>) -> Match<'a> {
    let completed_children = tree.children.iter().map(|c| complete_capture_tree(c));

    CaptureTree {
        name: tree.name,
        slice: tree.slice.complete(),
        children: completed_children.collect(),
    }
}

#[derive(Debug, Clone)]
struct Thread<'a, 'c> {
    instructions: &'a [Instruction],
    string: &'c [(String, u32)],
    program_pointer: usize,
    string_pointer: usize,
    rule_stack: Vec<u32>,
    call_stack: Vec<usize>,
    captures: Vec<CaptureTree<'a, Capture>>,
    progress: HashMap<usize, usize>,
}

impl<'a, 'c> Thread<'a, 'c> {
    fn new(instructions: &'a [Instruction], string: &'c [(String, u32)]) -> Self {
        Thread {
            instructions: instructions,
            string: string,
            program_pointer: 0,
            string_pointer: 0,
            rule_stack: Vec::new(),
            call_stack: Vec::new(),
            captures: Vec::new(),
            progress: HashMap::new(),
        }
    }

    fn capture_start(&mut self, name: CaptureName<'a>) {
        let position = self.string_pointer;

        self.captures
            .push(CaptureTree {
                      name: name,
                      slice: Capture::Started(position),
                      children: Vec::new(),
                  });
    }

    fn capture_stop(&mut self) -> Option<CaptureTree<'a, Capture>> {
        let position = self.string_pointer;

        let mut child = self.captures.pop().unwrap();
        if let Capture::Started(start) = child.slice {
            child.slice = Capture::Complete(start, position);
        } else {
            panic!("attempt to stop capture twice");
        }

        if let Some(parent) = self.captures.last_mut() {
            parent.children.push(child);
            None
        } else {
            Some(child)
        }
    }

    fn current_rule(&self) -> u32 {
        *self.rule_stack.last().unwrap()
    }

    fn run(mut self, threads: &mut Vec<Thread<'a, 'c>>) -> Option<Match<'a>> {
        loop {
            let next = &self.instructions[self.program_pointer];
            self.program_pointer += 1;

            match *next {
                Instruction::RuleStart(id, ref name) => {
                    self.rule_stack.push(id);
                    self.capture_start(CaptureName::Rule { name });
                }
                Instruction::Literal(ref grammar_word) => {
                    if let Some(&(ref word, id)) = self.string.get(self.string_pointer) {
                        if (word, id) == (grammar_word, self.current_rule()) {
                            self.string_pointer += 1;
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                Instruction::AnyWord => {
                    if let Some(&(_, id)) = self.string.get(self.string_pointer) {
                        if id == self.current_rule() {
                            self.string_pointer += 1;
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                Instruction::GreedyRule(rule) => {
                    while let Some(&(_, id)) = self.string.get(self.string_pointer) {
                        if id == rule {
                            self.string_pointer += 1;
                        } else {
                            break;
                        }
                    }
                }
                Instruction::RuleStop => {
                    let maybe_node = self.capture_stop();

                    self.rule_stack.pop();

                    if let Some(return_address) = self.call_stack.pop() {
                        self.program_pointer = return_address;
                    } else if self.string_pointer == self.string.len() {
                        return Some(complete_capture_tree(&maybe_node.unwrap()));
                    } else {
                        return None;
                    }
                }
                Instruction::CaptureStart(ref name) => {
                    self.capture_start(CaptureName::Capture { name });
                }
                Instruction::CaptureStop => {
                    self.capture_stop();
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

    fn perform_match(mut self) -> Option<Match<'a>> {
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
