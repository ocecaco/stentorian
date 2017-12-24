use components::comptr::ComPtr;
use components::{create_instance, query_interface, raw_to_comptr, ComInterface, IUnknown,
                 RawComPtr, CLSCTX_LOCAL_SERVER, GUID, HRESULT};
use interfaces::{CLSID_DgnDictate, CLSID_DgnSite, IDgnSREngineControl, IDgnSREngineNotifySink,
                 IDgnSRGramCommon, ISRCentral, ISRGramCommon, ISRGramNotifySink, ISRSpeaker,
                 IServiceProvider};
use std::ptr;
use std::mem;
use std::sync::{Arc, RwLock};
use dragon::SRGRMFMT;
use grammar::{Element, Grammar, Rule};
use grammarcompiler::{compile_command_grammar, compile_dictation_grammar, compile_select_grammar};
use self::enginesink::{EngineSink, PauseCookie};
use self::grammarsink::{GrammarSink, RawGrammarEvent};
use self::events::{Attribute, EngineEvent, GrammarEvent};
use self::results::{CommandGrammarEvent, SelectGrammarEvent};
use errors::*;

mod enginesink;
mod grammarsink;
mod grammarcontrol;
mod events;
mod results;

pub use self::grammarcontrol::{CatchallGrammarControl, CommandGrammarControl,
                               DictationGrammarControl, SelectGrammarControl};

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

    pub fn get_current_user(&self) -> Result<Option<String>> {
        const SIZE: usize = 128;
        const NO_USER_SELECTED: HRESULT = HRESULT(0x8004_041a);
        type Buffer = [u16; SIZE];

        let speaker = query_interface::<ISRSpeaker>(&self.central)?;
        let mut buffer: Buffer = [0u16; SIZE];
        let mut required = 0;

        let rc = unsafe {
            speaker.query(
                buffer.as_mut_ptr(),
                mem::size_of::<Buffer>() as u32,
                &mut required,
            )
        };

        if rc == NO_USER_SELECTED {
            return Ok(None);
        }

        try!(rc.result());
        Ok(Some(String::from_utf16_lossy(
            &buffer[..(required / 2 - 1) as usize],
        )))
    }

    pub fn register<F>(&self, callback: F) -> Result<EngineRegistration>
    where
        F: Fn(EngineEvent) + Sync + 'static,
    {
        let sink = EngineSink::create(
            engine_flags::SEND_PAUSED | engine_flags::SEND_ATTRIBUTE,
            Box::new(callback),
        );

        let mut key = 0;
        let rc = unsafe {
            self.central.register(
                &sink as &IUnknown as *const _ as RawComPtr,
                IDgnSREngineNotifySink::iid(),
                &mut key,
            )
        };

        try!(rc.result());

        let registration = EngineRegistration {
            central: self.central.clone(),
            register_key: key,
        };

        Ok(registration)
    }

    fn grammar_helper<F>(
        &self,
        grammar_type: SRGRMFMT,
        compiled: &[u8],
        all_recognitions: bool,
        callback: F,
    ) -> Result<ComPtr<ISRGramCommon>>
    where
        F: Fn(RawGrammarEvent) + Sync + 'static,
    {
        let mut raw_control = ptr::null();

        let mut flags = grammar_flags::SEND_PHRASE_START | grammar_flags::SEND_PHRASE_FINISH;

        if all_recognitions {
            flags |= grammar_flags::SEND_FOREIGN_FINISH;
        }

        let sink = GrammarSink::create(flags, Box::new(callback));
        let raw_sink = &sink as &IUnknown as *const _ as RawComPtr;

        let rc = unsafe {
            self.central.grammar_load(
                grammar_type,
                compiled.into(),
                raw_sink,
                ISRGramNotifySink::iid(),
                &mut raw_control,
            )
        };

        try!(rc.result());

        let grammar_control = unsafe { raw_to_comptr::<IUnknown>(raw_control, true) };
        let grammar_control = query_interface::<ISRGramCommon>(&grammar_control)?;

        Ok(grammar_control)
    }

    pub fn select_grammar_load<F>(
        &self,
        select_words: &[String],
        through_words: &[String],
        callback: F,
    ) -> Result<SelectGrammarControl>
    where
        F: Fn(SelectGrammarEvent) + Sync + 'static,
    {
        let compiled = compile_select_grammar(select_words, through_words);

        let guid_field = Arc::new(RwLock::new(None));
        let guid_clone = Arc::clone(&guid_field);

        let wrapped = move |e: RawGrammarEvent| {
            let guid = guid_clone.read().unwrap().unwrap();
            let new_event = e.map(|r| results::retrieve_selection_choices(&r.ptr, guid).unwrap());
            callback(new_event);
        };

        let control =
            self.grammar_helper(SRGRMFMT::SRGRMFMT_DRAGONNATIVE1, &compiled, false, wrapped)?;

        let guid = grammar_guid(&control)?;
        let mut guid_data = guid_field.write().unwrap();
        *guid_data = Some(guid);

        SelectGrammarControl::create(control)
    }

    pub fn command_grammar_load<F>(
        &self,
        grammar: &Grammar,
        callback: F,
    ) -> Result<CommandGrammarControl>
    where
        F: Fn(CommandGrammarEvent) + Sync + 'static,
    {
        let compiled = compile_command_grammar(grammar)?;

        let wrapped = move |e: RawGrammarEvent| {
            let new_event = e.map(|r| results::retrieve_command_choices(&r.ptr).unwrap());
            callback(new_event);
        };
        let control = self.grammar_helper(SRGRMFMT::SRGRMFMT_CFG, &compiled, false, wrapped)?;

        CommandGrammarControl::create(control)
    }

    pub fn dictation_grammar_load<F>(&self, callback: F) -> Result<DictationGrammarControl>
    where
        F: Fn(CommandGrammarEvent) + Sync + 'static,
    {
        let compiled = compile_dictation_grammar();
        let wrapped = move |e: RawGrammarEvent| {
            let new_event = e.map(|r| results::retrieve_command_choices(&r.ptr).unwrap());
            callback(new_event);
        };
        let control = self.grammar_helper(SRGRMFMT::SRGRMFMT_CFG, &compiled, false, wrapped)?;

        DictationGrammarControl::create(control)
    }

    pub fn catchall_grammar_load<F>(&self, callback: F) -> Result<CatchallGrammarControl>
    where
        F: Fn(CommandGrammarEvent) + Sync + 'static,
    {
        let rule = Rule {
            name: "".to_owned(),
            exported: true,
            definition: Element::List {
                name: "_impossible".to_owned(),
            },
        };

        let grammar = Grammar { rules: vec![rule] };
        let compiled = compile_command_grammar(&grammar)?;

        let wrapped = move |e: RawGrammarEvent| {
            let new_event = e.map(|r| results::retrieve_command_choices(&r.ptr).unwrap());
            callback(new_event);
        };
        let control = self.grammar_helper(SRGRMFMT::SRGRMFMT_CFG, &compiled, true, wrapped)?;

        CatchallGrammarControl::create(control)
    }
}

pub struct EngineRegistration {
    central: ComPtr<ISRCentral>,
    register_key: u32,
}

impl Drop for EngineRegistration {
    fn drop(&mut self) {
        unsafe {
            let rc = self.central.unregister(self.register_key);
            rc.result().expect("engine unregister failed");
        }
    }
}

fn grammar_guid(grammar_control: &IUnknown) -> Result<GUID> {
    let grammar_dragon = query_interface::<IDgnSRGramCommon>(grammar_control)?;

    let mut guid: GUID = unsafe { mem::uninitialized() };
    let rc = unsafe { grammar_dragon.identify(&mut guid) };
    rc.result()?;

    Ok(guid)
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
