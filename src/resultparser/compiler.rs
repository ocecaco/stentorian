use std::collections::HashMap;
use grammar::*;
use super::instructions::*;

pub fn compile_grammar_matcher(grammar: &Grammar) -> Vec<Instruction> {
    let compiler = Compiler::new();
    let mut instructions = compiler.compile_rules(&grammar.rules);
    let locations = find_label_locations(&instructions);
    relabel(&mut instructions, &locations);
    instructions
}


fn find_label_locations(instructions: &[Instruction]) -> HashMap<LabelName, usize> {
    let mut locations = HashMap::new();

    for (i, ins) in instructions.iter().enumerate() {
        if let Instruction::Label(name) = *ins {
            // TODO: check for duplicates
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
            Instruction::Jump(ref mut target) => {
                relabel_target(target, locations);
            }
            Instruction::Split(ref mut targets) => {
                for t in targets.iter_mut() {
                    relabel_target(t, locations);
                }
            }
            Instruction::RuleCall(ref mut target) => {
                relabel_target(target, locations);
            }
            Instruction::Label(_) => {
                *i = Instruction::NoOp;
            }
            _ => ()
        }
    }
}

struct Compiler {
    rule_name_to_label: HashMap<String, (u32, Option<LabelName>)>,
    label_counter: u32,
    instructions: Vec<Instruction>
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            rule_name_to_label: HashMap::new(),
            label_counter: 0,
            instructions: Vec::new()
        }
    }

    fn emit(&mut self, i: Instruction) {
        self.instructions.push(i);
    }

    fn new_label(&mut self) -> LabelName {
        self.label_counter += 1;

        LabelName(self.label_counter)
    }

    fn compile_rules(mut self, rules: &[(String, Rule)]) -> Vec<Instruction> {
        let mut with_labels = Vec::new();
        let mut labels = Vec::new();
        for (i, &(ref name, ref r)) in (1u32..).zip(rules.iter()) {
            match *r {
                Rule::DefinedRule(RuleVisibility::Exported, _) => {
                    let n = self.new_label();
                    labels.push(n);
                    with_labels.push((i, name, r, Some(n)));
                }
                Rule::DefinedRule(RuleVisibility::Local, _) => {
                    let n = self.new_label();
                    with_labels.push((i, name, r, Some(n)));
                }
                Rule::ImportedRule => with_labels.push((i, name, r, None))
            }
        }

        let labels = labels.iter()
            .map(|&n| JumpTarget::Symbolic(n))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self.emit(Instruction::Split(labels));

        for &(i, name, r, label) in &with_labels {
            self.rule_name_to_label.insert(name.clone(), (i, label));

            if let Rule::DefinedRule(ref visibility, ref element) = *r {
                self.compile_single_rule(i, *visibility, element, label);
            }
        }

        self.instructions
    }

    fn compile_single_rule(&mut self,
                           rule_id: u32,
                           visibility: RuleVisibility,
                           element: &Element,
                           start_label: Option<LabelName>) {
        if let Some(n) = start_label {
            self.emit(Instruction::Label(n));
        }

        match visibility {
            RuleVisibility::Exported => self.emit(Instruction::TopLevelRule(rule_id)),
            RuleVisibility::Local => ()
        }
        self.compile_element(element);
        self.emit(Instruction::Match);
    }

    fn compile_element(&mut self, element: &Element) {
        match *element {
            Element::Sequence(ref children) => {
                for c in children.iter() {
                    self.compile_element(c);
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

                self.emit(Instruction::Split(jump_targets));

                let end = self.new_label();
                for (start, c) in labels.iter().zip(children.iter()) {
                    self.emit(Instruction::Label(*start));

                    self.compile_element(c);

                    self.emit(Instruction::Jump(JumpTarget::Symbolic(end)));
                }

                self.emit(Instruction::Label(end));
            }
            Element::Repetition(ref child) => {
                let loop_label = self.new_label();
                let child_label = self.new_label();
                let done_label = self.new_label();

                self.emit(Instruction::Label(loop_label));

                let targets = vec![JumpTarget::Symbolic(child_label),
                                   JumpTarget::Symbolic(done_label)];
                let targets = targets.into_boxed_slice();
                self.emit(Instruction::Split(targets));

                self.emit(Instruction::Label(child_label));

                self.compile_element(child);

                self.emit(Instruction::Jump(JumpTarget::Symbolic(loop_label)));

                self.emit(Instruction::Label(done_label));
            }
            Element::Optional(ref child) => {
                let yes_label = self.new_label();
                let no_label = self.new_label();

                let targets = vec![JumpTarget::Symbolic(yes_label),
                                   JumpTarget::Symbolic(no_label)];
                let targets = targets.into_boxed_slice();
                self.emit(Instruction::Split(targets));
                self.emit(Instruction::Label(yes_label));

                self.compile_element(child);

                self.emit(Instruction::Label(no_label));
            }
            Element::Capture(ref name, ref child) => {
                self.emit(Instruction::CaptureStart(name.clone()));
                self.compile_element(child);
                self.emit(Instruction::CaptureStop(name.clone()));
            }
            Element::Literal(ref word) => {
                self.emit(Instruction::Literal(word.clone()));
            }
            Element::Rule(ref name) => {
                let target = JumpTarget::Symbolic(self.rule_name_to_label[name].1.unwrap());
                self.emit(Instruction::RuleCall(target));
            }
            Element::List(ref name) => {
                self.emit(Instruction::List(name.clone()));
            }
        }
    }
}

