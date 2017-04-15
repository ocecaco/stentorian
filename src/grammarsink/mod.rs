mod grammarsink;
mod interfaces;

bitflags! {
    pub flags GrammarSinkFlags: u32 {
        const SEND_PHRASE_START = 0x1000,
        const SEND_PHRASE_HYPOTHESIS = 0x2000,
        const SEND_PHRASE_FINISH = 0x4000,
        const SEND_FOREIGN_FINISH = 0x8000,
    }
}

pub enum GrammarEvent {
    Bookmark,
    Paused,
    PhraseFinish(Box<[(String, u32)]>),
    PhraseHypothesis,
    PhraseStart,
    Reevaluate,
    Training,
    Unarchive,
}
