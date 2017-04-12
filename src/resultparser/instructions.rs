#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct LabelName(pub u32);

#[derive(Debug, Copy, Clone)]
pub enum JumpTarget {
    Symbolic(LabelName),
    Concrete(usize),
}

#[derive(Debug, Clone)]
pub enum Instruction {
    TopLevelRule(u32),
    Literal(String),
    List(String),
    Match,
    Label(LabelName),
    NoOp,
    CaptureStart(String),
    CaptureStop(String),
    RuleCall(JumpTarget),
    Jump(JumpTarget),
    Split(Box<[JumpTarget]>),
}
