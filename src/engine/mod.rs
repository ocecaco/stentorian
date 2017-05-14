use components::comptr::ComPtr;
use components::bstr::BString;
use components::*;
use interfaces::*;
use self::interfaces::*;
use std::ptr;
use std::mem;
use dragon::*;
use grammar::Grammar;
use self::enginesink::*;
use self::grammarsink::*;
use self::grammarsink::interfaces::{ISRGramCommon, ISRGramNotifySink, ISRGramCFG};
use grammarcompiler::compile_grammar;
use errors::*;

mod interfaces;
mod enginesink;
mod grammarsink;

pub use self::enginesink::PauseCookie;
pub use self::enginesink::Callback as EngineCallback;
pub use self::grammarsink::Callback as GrammarCallback;

mod engine_flags {
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

            const SEND_ALL = 0x3ff,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Attribute {
    MicrophoneState,
}

#[derive(Debug)]
pub enum EngineEvent {
    AttributeChanged(Attribute),
    Paused(PauseCookie),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
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
    pub fn connect() -> Result<Self> {
        let central = get_central()?;
        let engine_control = query_interface::<IDgnSREngineControl>(&central)?;
        Ok(Engine {
            central: central,
            engine_control: engine_control,
        })
    }

    pub fn resume(&self, cookie: PauseCookie) -> Result<()> {
        let rc = unsafe { self.engine_control.resume(cookie.into()) };

        try!(rc.result());
        Ok(())
    }

    pub fn microphone_set_state(&self, state: MicrophoneState) -> Result<()> {
        let rc = unsafe { self.engine_control.set_mic_state(state as u16, 0) };

        try!(rc.result());
        Ok(())
    }

    pub fn microphone_get_state(&self) -> Result<MicrophoneState> {
        let mut state = MicrophoneState::Off;

        let rc = unsafe {
            let ptr = &mut state as *mut MicrophoneState as *mut u16;
            self.engine_control.get_mic_state(ptr)
        };

        try!(rc.result());
        Ok(state)
    }

    pub fn register(&self, callback: EngineCallback) -> Result<EngineRegistration>
    {
        let sink = EngineSink::create(engine_flags::SEND_PAUSED |
                                      engine_flags::SEND_ATTRIBUTE,
                                      callback);

        let mut key = 0;
        let rc = unsafe {
            self.central
                .register(&sink as &IUnknown as *const _ as RawComPtr,
                          IDgnSREngineNotifySink::iid(),
                          &mut key)
        };

        try!(rc.result());

        let registration = EngineRegistration {
            central: self.central.clone(),
            register_key: key,
        };

        Ok(registration)
    }

    pub fn grammar_load(&self,
                        grammar: &Grammar,
                        all_recognitions: bool,
                        callback: GrammarCallback)
                        -> Result<GrammarControl>
    {
        let compiled = compile_grammar(grammar)?;
        let data = SDATA {
            data: compiled.as_ptr(),
            size: compiled.len() as u32,
        };
        let mut raw_control = ptr::null();

        let mut flags = grammar_flags::SEND_PHRASE_START
            | grammar_flags::SEND_PHRASE_FINISH;

        if all_recognitions {
            flags |= grammar_flags::SEND_FOREIGN_FINISH;
        }

        let sink = GrammarSink::create(flags, callback);
        let raw_sink = &sink as &IUnknown as *const _ as RawComPtr;

        let rc = unsafe {
            self.central
                .grammar_load(SRGRMFMT::SRGRMFMT_CFG,
                              data,
                              raw_sink,
                              ISRGramNotifySink::iid(),
                              &mut raw_control)
        };

        try!(rc.result());

        let grammar_control = unsafe { raw_to_comptr::<IUnknown>(raw_control, true) };

        let grammar_control = query_interface::<ISRGramCommon>(&grammar_control)?;
        let grammar_lists = query_interface::<ISRGramCFG>(&grammar_control)?;

        let control = GrammarControl {
            grammar_control: grammar_control,
            grammar_lists: grammar_lists,
        };

        Ok(control)
    }
}

mod grammar_flags {
    bitflags! {
        pub flags GrammarSinkFlags: u32 {
            const SEND_PHRASE_START = 0x1000,
            const SEND_PHRASE_HYPOTHESIS = 0x2000,
            const SEND_PHRASE_FINISH = 0x4000,
            const SEND_FOREIGN_FINISH = 0x8000,
        }
    }
}

type Words = Box<[(String, u32)]>;

#[derive(Debug)]
pub struct Recognition {
    pub foreign: bool,
    pub words: Words,
}

#[derive(Debug)]
pub enum GrammarEvent {
    PhraseFinish(Option<Recognition>),
    PhraseStart,
}

pub struct GrammarControl {
    grammar_control: ComPtr<ISRGramCommon>,
    grammar_lists: ComPtr<ISRGramCFG>,
}

impl GrammarControl {
    pub fn rule_activate(&self, name: &str) -> Result<()> {
        let rc = unsafe {
            self.grammar_control
                .activate(ptr::null(), 0, BString::from(name).as_ref())
        };

        try!(rc.result());
        Ok(())
    }

    pub fn rule_deactivate(&self, name: &str) -> Result<()> {
        let rc = unsafe {
            self.grammar_control
                .deactivate(BString::from(name).as_ref())
        };

        try!(rc.result());
        Ok(())
    }

    pub fn list_append(&self, name: &str, word: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = word.into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { self.grammar_lists.list_append(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }

    pub fn list_remove(&self, name: &str, word: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = word.into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { self.grammar_lists.list_remove(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }

    pub fn list_clear(&self, name: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = "".into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { self.grammar_lists.list_set(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
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
            assert_eq!(result, S_OK, "engine unregister failed: {}", result);
        }
    }
}

fn get_central() -> Result<ComPtr<ISRCentral>> {
    let provider = create_instance::<IServiceProvider>(&CLSID_DgnSite, None, CLSCTX_LOCAL_SERVER)?;
    let mut central: RawComPtr = ptr::null();
    unsafe {
        let rc = provider.query_service(&CLSID_DgnDictate, &ISRCentral::iid(), &mut central);
        try!(rc.result());
        Ok(raw_to_comptr::<ISRCentral>(central, true))
    }
}
