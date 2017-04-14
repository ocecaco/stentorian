use super::enginesink::PauseCookie;

bitflags! {
    pub flags EngineSinkFlags: u32 {
        const SEND_BEGIN_UTTERANCE = 0x01,
        const SEND_END_UTTERANCE = 0x01,
        const SEND_VU_METER = 0x01,
        const SEND_ATTRIBUTE = 0x01,
        const SEND_INTERFERENCE = 0x01,
        const SEND_SOUND = 0x01,
        const SEND_PAUSED = 0x01,
        const SEND_ERROR = 0x01,
        const SEND_PROGRESS = 0x01,
        const SEND_MIMIC_DONE = 0x01,
    }
}

#[derive(Debug)]
pub enum EngineEvent {
    AttributeChanged,
    Interference,
    Sound,
    UtteranceBegin,
    UtteranceEnd,
    VuMeter,
    Paused(PauseCookie),
    MimicDone,
    ErrorHappened,
    Progress,
}
