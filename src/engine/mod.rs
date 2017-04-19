use std::sync::mpsc::Sender;
use components::comptr::ComPtr;
use components::bstr::BString;
use components::*;
use interfaces::*;
use self::interfaces::*;
use std::ptr;
use dragon::*;
use grammar::Grammar;
use self::enginesink::*;
use self::grammarsink::*;
use self::grammarsink::interfaces::{ISRGramCommon, ISRGramNotifySink};
use self::grammarcompiler::compile_grammar;

mod interfaces;
mod enginesink;
mod grammarsink;
mod grammarcompiler;
mod events;

pub use self::enginesink::PauseCookie;

bitflags! {
    pub flags EngineSinkFlags: u32 {
        const SEND_BEGIN_UTTERANCE = 0x01,
        const SEND_END_UTTERANCE = 0x02,
        const SEND_VU_METER = 0x04,
        const SEND_ATTRIBUTE = 0x08,
        const SEND_INTERFERENCE = 0x10,
        const SEND_SOUND = 0x20,
        const SEND_PAUSED = 0x40,
        const SEND_ERROR = 0x80,
        const SEND_PROGRESS = 0x100,
        const SEND_MIMIC_DONE = 0x200,
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Attribute {
    AutoGainEnable = 1,
    Threshold,
    Echo,
    EnergyFloor,
    Microphone,
    RealTime,
    Speaker,
    Timeout,
    StartListening,
    StopListening,

    MicrophoneState = 1001,
    Registry,
    PlaybackDone,
    Topic,
    LexiconAdd,
    LexiconRemove,
}

#[derive(Debug)]
pub enum EngineEvent {
    AttributeChanged(Attribute),
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum MicrophoneState {
    Disabled = 0,
    Off = 1,
    On = 2,
    Sleeping = 3,
}

pub struct Engine {
    central: ComPtr<ISRCentral>,
    engine_control: ComPtr<IDgnSREngineControl>,
}

impl Engine {
    pub fn connect() -> Self {
        let central = get_central();
        let engine_control = query_interface::<IDgnSREngineControl>(&central).unwrap();
        Engine {
            central: get_central(),
            engine_control: engine_control,
        }
    }

    pub fn resume(&self, cookie: PauseCookie) {
        unsafe {
            let result = self.engine_control.resume(cookie.into());
            assert_eq!(result.0, 0);
        }
    }

    pub fn register<T>(&self, flags: EngineSinkFlags, sender: Sender<T>) -> EngineRegistration
        where T: From<EngineEvent> + Send + 'static
    {
        let sink = EngineSink::create(flags, sender);
        let mut key = 0;
        unsafe {
            let result = self.central
                .register(&sink as &IUnknown as *const _ as RawComPtr,
                          IDgnSREngineNotifySink::iid(),
                          &mut key);
            assert_eq!(result.0, 0);
        }

        EngineRegistration {
            central: self.central.clone(),
            register_key: key,
        }
    }

    pub fn grammar_load<T>(&self,
                           flags: GrammarSinkFlags,
                           grammar: &Grammar,
                           sender: Sender<T>)
                           -> GrammarControl
        where T: From<GrammarEvent> + Send + 'static
    {
        let compiled = compile_grammar(grammar);
        let data = SDATA {
            data: compiled.as_ptr(),
            size: compiled.len() as u32,
        };
        let mut raw_control = ptr::null();

        let sink = GrammarSink::create(flags, sender);
        let raw_sink = &sink as &IUnknown as *const _ as RawComPtr;

        let grammar_control = unsafe {
            let result = self.central
                .grammar_load(SRGRMFMT::SRGRMFMT_CFG,
                              data,
                              raw_sink,
                              ISRGramNotifySink::iid(),
                              &mut raw_control);
            assert_eq!(result.0, 0);
            raw_to_comptr::<IUnknown>(raw_control, true)
        };

        let grammar_control = query_interface::<ISRGramCommon>(&grammar_control).unwrap();

        GrammarControl { grammar_control: grammar_control }
    }
}

bitflags! {
    pub flags GrammarSinkFlags: u32 {
        const SEND_PHRASE_START = 0x1000,
        const SEND_PHRASE_HYPOTHESIS = 0x2000,
        const SEND_PHRASE_FINISH = 0x4000,
        const SEND_FOREIGN_FINISH = 0x8000,
    }
}

#[derive(Debug)]
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

pub struct GrammarControl {
    grammar_control: ComPtr<ISRGramCommon>,
}

impl GrammarControl {
    pub fn activate_rule(&self, name: &str) {
        unsafe {
            let result = self.grammar_control
                .activate(ptr::null(), 0, BString::from(name).as_ref());
            assert_eq!(result.0, 0);
        }
    }

    pub fn deactivate_rule(&self, name: &str) {
        unsafe {
            let result = self.grammar_control
                .deactivate(BString::from(name).as_ref());
            assert_eq!(result.0, 0);
        }
    }
}

pub struct EngineRegistration {
    central: ComPtr<ISRCentral>,
    register_key: u32,
}

impl Drop for EngineRegistration {
    fn drop(&mut self) {
        unsafe {
            let result = self.central.unregister(self.register_key);
            assert_eq!(result.0, 0);
        }
    }
}

fn get_central() -> ComPtr<ISRCentral> {
    let provider = create_instance::<IServiceProvider>(&CLSID_DgnSite, None, CLSCTX_LOCAL_SERVER)
        .unwrap();
    unsafe {
        let mut central: RawComPtr = ptr::null();
        let result = provider.query_service(&CLSID_DgnDictate, &ISRCentral::iid(), &mut central);
        assert_eq!(result.0, 0);
        raw_to_comptr::<ISRCentral>(central, true)
    }
}
