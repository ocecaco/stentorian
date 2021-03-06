use super::instructions::{Instruction, JumpTarget, LabelName};
use crate::grammar::{Element, Grammar, Rule};
use std::collections::HashMap;

pub fn compile_matcher(grammar: &Grammar) -> Vec<Instruction> {
    let compiler = Compiler::new();
    let mut instructions = compiler.compile_grammar(grammar);
    let locations = find_label_locations(&instructions);
    relabel(&mut instructions, &locations);
    instructions
}

fn find_label_locations(instructions: &[Instruction]) -> HashMap<LabelName, usize> {
    let mut locations = HashMap::new();

    for (i, ins) in instructions.iter().enumerate() {
        if let Instruction::Label(name) = *ins {
            locations.insert(name, i);
        }
    }

    locations
}

fn relabel(instructions: &mut [Instruction], locations: &HashMap<LabelName, usize>) {
    fn relabel_target(t: &mut JumpTarget, locations: &HashMap<LabelName, usize>) {
        if let JumpTarget::Symbolic(name) = *t {
            *t = JumpTarget::Concrete(locations[&name]);
        }
    }

    for i in instructions.iter_mut() {
        match *i {
            Instruction::Jump(ref mut target) | Instruction::RuleCall(ref mut target) => {
                relabel_target(target, locations);
            }
            Instruction::Split(ref mut targets) => {
                for t in targets.iter_mut() {
                    relabel_target(t, locations);
                }
            }
            Instruction::Label(_) => {
                *i = Instruction::NoOp;
            }
            _ => (),
        }
    }
}

fn make_split(labels: &[LabelName]) -> Instruction {
    let split = labels
        .iter()
        .map(|&n| JumpTarget::Symbolic(n))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Instruction::Split(split)
}

struct Compiler<'a> {
    rule_name_to_label: HashMap<&'a str, LabelName>,
    label_counter: u32,
    instructions: Vec<Instruction>,
}

impl<'a> Compiler<'a> {
    fn new() -> Self {
        Compiler {
            rule_name_to_label: HashMap::new(),
            label_counter: 0,
            instructions: Vec::new(),
        }
    }

    fn emit(&mut self, i: Instruction) {
        self.instructions.push(i);
    }

    fn new_label(&mut self) -> LabelName {
        self.label_counter += 1;

        LabelName(self.label_counter)
    }

    fn compile_grammar(mut self, grammar: &'a Grammar) -> Vec<Instruction> {
        let mut with_labels = Vec::new();
        let mut split_labels = Vec::new();
        for r in grammar.rules.iter() {
            let n = self.new_label();
            if r.exported {
                split_labels.push(n);
            }
            with_labels.push((r, n));
            self.rule_name_to_label.insert(&r.name, n);
        }

        self.emit(make_split(&split_labels));

        for &(r, label) in &with_labels {
            self.compile_single_rule(r, label);
        }

        self.instructions
    }

    fn compile_single_rule(&mut self, rule: &'a Rule, start_label: LabelName) {
        self.emit(Instruction::Label(start_label));
        self.compile_element(&rule.definition);
        self.emit(Instruction::Return);
    }

    fn compile_element(&mut self, element: &'a Element) {
        match *element {
            Element::Sequence { ref children } => {
                for c in children.iter() {
                    self.compile_element(c);
                }
            }
            Element::Alternative { ref children } => {
                let mut labels = Vec::new();
                for _ in 0..children.len() {
                    labels.push(self.new_label());
                }

                self.emit(make_split(&labels));

                let end = self.new_label();
                for (start, c) in labels.iter().zip(children.iter()) {
                    self.emit(Instruction::Label(*start));

                    self.compile_element(c);

                    self.emit(Instruction::Jump(JumpTarget::Symbolic(end)));
                }

                self.emit(Instruction::Label(end));
            }
            Element::Repetition { ref child } => {
                let loop_label = self.new_label();
                let done_label = self.new_label();

                self.emit(Instruction::Label(loop_label));

                self.emit(Instruction::Progress);

                self.compile_element(child);

                self.emit(make_split(&[loop_label, done_label]));

                self.emit(Instruction::Label(done_label));
            }
            Element::Optional { ref child } => {
                let yes_label = self.new_label();
                let no_label = self.new_label();

                self.emit(make_split(&[yes_label, no_label]));
                self.emit(Instruction::Label(yes_label));

                self.compile_element(child);

                self.emit(Instruction::Label(no_label));
            }
            Element::Capture {
                ref name,
                ref child,
            } => {
                self.emit(Instruction::CaptureStart(name.clone()));
                self.compile_element(child);
                self.emit(Instruction::CaptureStop);
            }
            Element::Word { ref text } => {
                self.emit(Instruction::Literal(text.clone()));
            }
            Element::RuleRef { ref name } => {
                let name: &'a str = name;
                let target = JumpTarget::Symbolic(self.rule_name_to_label[name]);
                self.emit(Instruction::RuleCall(target));
            }
            Element::List { .. } | Element::DictationWord | Element::SpellingLetter => {
                self.emit(Instruction::AnyWord);
            }
            Element::Dictation => {
                let loop_label = self.new_label();
                let done_label = self.new_label();

                self.emit(Instruction::Label(loop_label));

                self.emit(Instruction::Progress);

                self.emit(Instruction::AnyWord);

                // non-greedy since done label is put first
                self.emit(make_split(&[done_label, loop_label]));

                self.emit(Instruction::Label(done_label));
            }
        }
    }
}
