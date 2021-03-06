use self::enginesink::{EngineSink, PauseCookie};
use self::grammarsink::{GrammarSink, RawGrammarEvent};
use crate::dragon::SRGRMFMT;
use crate::errors::*;
use crate::grammar::{Element, Grammar, Rule};
use crate::grammarcompiler::{
    compile_command_grammar, compile_dictation_grammar, compile_select_grammar,
};
use crate::interfaces::{
    CLSID_DgnDictate, CLSID_DgnSite, IDgnSREngineControl, IDgnSREngineNotifySink, IDgnSRGramCommon,
    ISRCentral, ISRGramCommon, ISRGramNotifySink, ISRSpeaker, IServiceProvider,
};
use bitflags::bitflags;
use components::comptr::ComPtr;
use components::{
    create_instance, raw_to_comptr, Cast, ComInterface, IUnknown, RawComPtr, CLSCTX_LOCAL_SERVER,
    GUID, HRESULT,
};
use serde::{Deserialize, Serialize};
use std::mem;
use std::ptr;
use std::sync::{Arc, RwLock};

mod enginesink;
mod events;
mod grammarcontrol;
mod grammarsink;
mod results;

pub use self::events::{EngineEvent, GrammarEvent};
pub use self::grammarcontrol::{
    CatchallGrammarControl, CommandGrammarControl, DictationGrammarControl, SelectGrammarControl,
};
pub use self::results::{
    CatchallGrammarEvent, CommandGrammarEvent, DictationGrammarEvent, SelectGrammarEvent, WordInfo,
    Words,
};

bitflags! {
    pub struct GrammarFlags: u32 {
        const SEND_PHRASE_START = 0x1000;
        const SEND_PHRASE_HYPOTHESIS = 0x2000;
        const SEND_PHRASE_FINISH = 0x4000;
        const SEND_FOREIGN_FINISH = 0x8000;
    }
}

bitflags! {
    pub struct EngineFlags: u32 {
        const SEND_BEGIN_UTTERANCE = 0x01;
        const SEND_END_UTTERANCE = 0x02;
        const SEND_VU_METER = 0x04;
        const SEND_ATTRIBUTE = 0x08;
        const SEND_INTERFERENCE = 0x10;
        const SEND_SOUND = 0x20;
        const SEND_PAUSED = 0x40;
        const SEND_ERROR = 0x80;
        const SEND_PROGRESS = 0x100;
        const SEND_MIMIC_DONE = 0x200;

        const SEND_ALL = 0x3ff;
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
        let engine_control = central.cast()?;
        Ok(Engine {
            central: central,
            engine_control: engine_control,
        })
    }

    pub fn resume(&self, cookie: PauseCookie) -> Result<()> {
        let rc = unsafe { self.engine_control.resume(cookie.into()) };

        rc.result()?;
        Ok(())
    }

    pub fn microphone_set_state(&self, state: MicrophoneState) -> Result<()> {
        let rc = unsafe { self.engine_control.set_mic_state(state as u16, 0) };

        rc.result()?;
        Ok(())
    }

    pub fn microphone_get_state(&self) -> Result<MicrophoneState> {
        let mut state = MicrophoneState::Off;

        let rc = unsafe {
            let ptr = &mut state as *mut MicrophoneState as *mut u16;
            self.engine_control.get_mic_state(ptr)
        };

        rc.result()?;
        Ok(state)
    }

    pub fn get_current_user(&self) -> Result<Option<String>> {
        const SIZE: usize = 128;
        const NO_USER_SELECTED: HRESULT = HRESULT(0x8004_041a);
        type Buffer = [u16; SIZE];

        let speaker = self.central.cast::<ISRSpeaker>()?;
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

        rc.result()?;
        Ok(Some(String::from_utf16_lossy(
            &buffer[..(required / 2 - 1) as usize],
        )))
    }

    pub fn register<F>(&self, callback: F) -> Result<EngineRegistration>
    where
        F: Fn(EngineEvent) + Sync + 'static,
    {
        let sink = EngineSink::create(
            EngineFlags::SEND_PAUSED | EngineFlags::SEND_ATTRIBUTE,
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

        rc.result()?;

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
        let mut raw_control = ptr::null_mut();

        let mut flags = GrammarFlags::SEND_PHRASE_START | GrammarFlags::SEND_PHRASE_FINISH;

        if all_recognitions {
            flags |= GrammarFlags::SEND_FOREIGN_FINISH;
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

        rc.result()?;

        let grammar_control = unsafe { raw_to_comptr::<IUnknown>(raw_control, true) };
        let grammar_control = grammar_control.cast()?;

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
            let new_event = e.map(|r| {
                results::retrieve_selection_choices(&r.ptr.cast().unwrap(), guid).unwrap()
            });
            callback(new_event);
        };

        let control =
            self.grammar_helper(SRGRMFMT::SRGRMFMT_DRAGONNATIVE1, &compiled, false, wrapped)?;

        let grammar_dragon = control.cast()?;
        let guid = grammar_guid(&grammar_dragon)?;
        let mut guid_data = guid_field.write().unwrap();
        *guid_data = Some(guid);

        grammarcontrol::create_select(control)
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
            let new_event = e.map(|r| {
                results::retrieve_words(&r.ptr.cast().unwrap(), 0)
                    .unwrap()
                    .unwrap()
            });
            callback(new_event);
        };
        let control = self.grammar_helper(SRGRMFMT::SRGRMFMT_CFG, &compiled, false, wrapped)?;

        grammarcontrol::create_command(control)
    }

    pub fn dictation_grammar_load<F>(&self, callback: F) -> Result<DictationGrammarControl>
    where
        F: Fn(DictationGrammarEvent) + Sync + 'static,
    {
        let compiled = compile_dictation_grammar();
        let wrapped = move |e: RawGrammarEvent| {
            let new_event =
                e.map(|r| results::retrieve_command_choices(&r.ptr.cast().unwrap()).unwrap());
            callback(new_event);
        };
        let control =
            self.grammar_helper(SRGRMFMT::SRGRMFMT_DICTATION, &compiled, false, wrapped)?;

        grammarcontrol::create_dictation(control)
    }

    pub fn catchall_grammar_load<F>(&self, callback: F) -> Result<CatchallGrammarControl>
    where
        F: Fn(CatchallGrammarEvent) + Sync + 'static,
    {
        let rule = Rule {
            name: "dummy".to_owned(),
            exported: true,
            definition: Element::List {
                name: "_impossible".to_owned(),
            },
        };

        let grammar = Grammar { rules: vec![rule] };
        let compiled = compile_command_grammar(&grammar)?;

        let wrapped = move |e: RawGrammarEvent| {
            let new_event =
                e.map(|r| results::retrieve_command_choices(&r.ptr.cast().unwrap()).unwrap());
            callback(new_event);
        };
        let control = self.grammar_helper(SRGRMFMT::SRGRMFMT_CFG, &compiled, true, wrapped)?;

        grammarcontrol::create_catchall(control)
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

fn grammar_guid(grammar_dragon: &IDgnSRGramCommon) -> Result<GUID> {
    let mut guid: GUID = GUID {
        data1: 0,
        data2: 0,
        data3: 0,
        data4: [0u8; 8],
    };

    let rc = unsafe { grammar_dragon.identify(&mut guid) };
    rc.result()?;

    Ok(guid)
}

fn get_central() -> Result<ComPtr<ISRCentral>> {
    let provider = create_instance::<IServiceProvider>(&CLSID_DgnSite, None, CLSCTX_LOCAL_SERVER)?;
    let mut central: RawComPtr = ptr::null_mut();
    unsafe {
        let rc = provider.query_service(&CLSID_DgnDictate, &ISRCentral::iid(), &mut central);
        rc.result()?;
        Ok(raw_to_comptr::<ISRCentral>(central, true))
    }
}
