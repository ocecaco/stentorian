#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct LabelName(pub u32);

#[derive(Debug, Copy, Clone)]
pub enum JumpTarget {
    Symbolic(LabelName),
    Concrete(usize),
}

impl JumpTarget {
    pub fn address(&self) -> usize {
        if let JumpTarget::Concrete(address) = *self {
            address
        } else {
            panic!("found symbolic jump instruction");
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Literal(String),
    AnyWord,

    Label(LabelName),
    NoOp,

    Progress,

    CaptureStart(String),
    CaptureStop,

    RuleCall(JumpTarget),
    Return,
    Jump(JumpTarget),
    Split(Box<[JumpTarget]>),
}
