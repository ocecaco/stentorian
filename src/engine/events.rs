use super::enginesink::PauseCookie;

#[derive(Debug)]
pub enum GrammarEvent<T> {
    PhraseFinish(T),
    PhraseRecognitionFailure,
    PhraseStart,
}

impl<T> GrammarEvent<T> {
    pub fn map<F, U>(self, f: F) -> GrammarEvent<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            GrammarEvent::PhraseFinish(v) => GrammarEvent::PhraseFinish(f(v)),
            GrammarEvent::PhraseRecognitionFailure => GrammarEvent::PhraseRecognitionFailure,
            GrammarEvent::PhraseStart => GrammarEvent::PhraseStart,
        }
    }
}

#[derive(Debug)]
pub enum Attribute {
    MicrophoneState,
}

#[derive(Debug)]
pub enum EngineEvent {
    AttributeChanged(Attribute),
    Paused(PauseCookie),
}
