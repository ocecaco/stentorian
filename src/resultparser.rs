use std::collections::HashMap;
use grammar::*;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct LabelName(u32);

#[derive(Debug, Copy, Clone)]
pub enum JumpTarget {
    Symbolic(LabelName),
    Concrete(usize)
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Literal(String),
    List(String),
    Match,
    Label(LabelName),
    NoOp,
    CaptureStart(String),
    CaptureStop(String),
    Rule(String),
    Jump(JumpTarget),
    Split(Box<[JumpTarget]>)
}

pub fn compile(element: &Element) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut compiler = Compiler::new();
    compiler.compile_matcher(&mut instructions, element);
    instructions.push(Instruction::Match);
    let locations = find_label_locations(&instructions);
    relabel(&mut instructions, &locations);
    instructions
}

fn find_label_locations(instructions: &[Instruction]) -> HashMap<LabelName, usize> {
    let mut locations = HashMap::new();
    
    for (i, ins) in instructions.iter().enumerate() {
        match *ins {
            Instruction::Label(name) => {
                // TODO: check for duplicates
                locations.insert(name, i);
            }
            _ => ()
        }
    }

    locations
}

fn relabel_target(t: &mut JumpTarget, locations: &HashMap<LabelName, usize>) {
    match t {
        &mut JumpTarget::Symbolic(name) => {
            *t = JumpTarget::Concrete(locations[&name]);
        }
        _ => ()
    }
}

fn relabel(instructions: &mut [Instruction], locations: &HashMap<LabelName, usize>) {
    for i in instructions.iter_mut() {
        match i {
            &mut Instruction::Jump(ref mut target) => {
                relabel_target(target, locations);
            }
            &mut Instruction::Split(ref mut targets) => {
                for t in targets.iter_mut() {
                    relabel_target(t, locations);
                }
            }
            &mut Instruction::Label(_) => {
                *i = Instruction::NoOp;
            }
            _ => ()
        }
    }
}

struct Compiler {
    label_counter: u32
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            label_counter: 0
        }
    }

    fn new_label(&mut self) -> LabelName {
        self.label_counter += 1;
        
        LabelName(self.label_counter)
    }

    fn compile_matcher(&mut self, output: &mut Vec<Instruction>, element: &Element) {
        match *element {
            Element::Sequence(ref children) => {
                for c in children.iter() {
                    self.compile_matcher(output, c);
                }
            }
            Element::Alternative(ref children) => {
                let mut labels = Vec::new();
                for _ in 0..children.len() {
                    labels.push(self.new_label());
                }

                let jump_targets = labels.iter()
                    .map(|&name| JumpTarget::Symbolic(name))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                
                output.push(Instruction::Split(jump_targets));

                let end = self.new_label();
                for (start, c) in labels.iter().zip(children.iter()) {
                    output.push(Instruction::Label(*start));

                    self.compile_matcher(output, c);

                    output.push(Instruction::Jump(JumpTarget::Symbolic(end)));
                }

                output.push(Instruction::Label(end));
            }
            Element::Repetition(ref child) => {
                let loop_label = self.new_label();
                let child_label = self.new_label();
                let done_label = self.new_label();

                output.push(Instruction::Label(loop_label));

                let targets = vec![JumpTarget::Symbolic(child_label),
                                   JumpTarget::Symbolic(done_label)];
                let targets = targets.into_boxed_slice();
                output.push(Instruction::Split(targets));

                output.push(Instruction::Label(child_label));

                self.compile_matcher(output, child);

                output.push(Instruction::Jump(JumpTarget::Symbolic(loop_label)));

                output.push(Instruction::Label(done_label));
            }
            Element::Optional(ref child) => {
                let yes_label = self.new_label();
                let no_label = self.new_label();

                let targets = vec![JumpTarget::Symbolic(yes_label),
                                   JumpTarget::Symbolic(no_label)];
                let targets = targets.into_boxed_slice();
                output.push(Instruction::Split(targets));
                output.push(Instruction::Label(yes_label));

                self.compile_matcher(output, child);
                
                output.push(Instruction::Label(no_label));
            }
            Element::Capture(ref name, ref child) => {
                output.push(Instruction::CaptureStart(name.clone()));
                self.compile_matcher(output, child);
                output.push(Instruction::CaptureStop(name.clone()));
            }
            Element::Literal(ref word) => {
                output.push(Instruction::Literal(word.clone()));
            }
            Element::Rule(ref name) => {
                output.push(Instruction::Rule(name.clone()));
            }
            Element::List(ref name) => {
                output.push(Instruction::List(name.clone()));
            }
        }
    }
}

