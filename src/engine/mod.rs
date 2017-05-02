use futures::sync::oneshot;
use futures::{Poll, Stream, Future};
use futures::stream::MergedItem;
use futures::future;
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
use std::sync::{Arc, Weak};
use errors::*;

mod interfaces;
mod enginesink;
mod grammarsink;

pub use self::enginesink::PauseCookie;

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

    pub fn register(&self) -> Result<(EngineRegistration, EngineReceiver)>
    {
        let (sink, receiver) = EngineSink::create(engine_flags::SEND_PAUSED |
                                                  engine_flags::SEND_ATTRIBUTE);
        let mut key = 0;
        let rc = unsafe {
            self.central
                .register(&sink as &IUnknown as *const _ as RawComPtr,
                          IDgnSREngineNotifySink::iid(),
                          &mut key)
        };

        try!(rc.result());

        let (tx, rx) = oneshot::channel();
        let event_stream = with_cancellation(receiver, rx.map_err(|_| ()));

        let registration = EngineRegistration {
            cancel: Cancel::new(tx),
        };

        let engine_receiver = EngineReceiver {
            central: self.central.clone(),
            register_key: key,
            receiver: event_stream,
        };

        Ok((registration, engine_receiver))
    }

    pub fn grammar_load(&self,
                        grammar: &Grammar,
                        all_recognitions: bool)
                        -> Result<(GrammarControl, GrammarReceiver)>
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

        let (sink, receiver) = GrammarSink::create(flags);
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

        let (tx, rx) = oneshot::channel();
        let event_stream = with_cancellation(receiver, rx.map_err(|_| ()));

        let pointers = Arc::new(GrammarPointers {
            grammar_control: grammar_control,
            grammar_lists: grammar_lists,
        });

        let control = GrammarControl {
            pointers: Arc::downgrade(&pointers),
            cancel: Cancel::new(tx),
        };

        let receiver = GrammarReceiver {
            pointers: Some(pointers),
            receiver: event_stream,
        };

        Ok((control, receiver))
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

struct GrammarPointers {
    grammar_control: ComPtr<ISRGramCommon>,
    grammar_lists: ComPtr<ISRGramCFG>,
}

pub struct GrammarControl {
    pointers: Weak<GrammarPointers>,
    cancel: Cancel,
}

type EventStream<T, E> = Box<Stream<Item=T, Error=E> + Send>;

pub struct GrammarReceiver {
    pointers: Option<Arc<GrammarPointers>>,
    receiver: EventStream<GrammarEvent, ()>,
}

impl Drop for GrammarReceiver {
    fn drop(&mut self) {
        // make sure pointers are dropped before receiver
        self.pointers = None;
    }
}

impl Stream for GrammarReceiver {
    type Item = GrammarEvent;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.receiver.poll()
    }
}

impl GrammarControl {
    fn get_pointers(&self) -> Result<Arc<GrammarPointers>> {
        self.pointers
            .upgrade()
            .ok_or(ErrorKind::GrammarGone.into())
    }

    pub fn rule_activate(&self, name: &str) -> Result<()> {
        let pointers = self.get_pointers()?;

        let rc = unsafe {
            pointers.grammar_control
                .activate(ptr::null(), 0, BString::from(name).as_ref())
        };

        try!(rc.result());
        Ok(())
    }

    pub fn rule_deactivate(&self, name: &str) -> Result<()> {
        let pointers = self.get_pointers()?;
        let rc = unsafe {
            pointers.grammar_control
                .deactivate(BString::from(name).as_ref())
        };

        try!(rc.result());
        Ok(())
    }

    pub fn list_append(&self, name: &str, word: &str) -> Result<()> {
        let pointers = self.get_pointers()?;
        let name = BString::from(name);
        let srword: SRWORD = word.into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { pointers.grammar_lists.list_append(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }

    pub fn list_remove(&self, name: &str, word: &str) -> Result<()> {
        let pointers = self.get_pointers()?;
        let name = BString::from(name);
        let srword: SRWORD = word.into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { pointers.grammar_lists.list_remove(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }

    pub fn list_clear(&self, name: &str) -> Result<()> {
        let pointers = self.get_pointers()?;
        let name = BString::from(name);
        let srword: SRWORD = "".into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { pointers.grammar_lists.list_set(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }
}


pub struct EngineRegistration {
    cancel: Cancel,
}

pub struct EngineReceiver {
    central: ComPtr<ISRCentral>,
    register_key: u32,
    receiver: EventStream<EngineEvent, ()>,
}

impl Stream for EngineReceiver {
    type Item = EngineEvent;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.receiver.poll()
    }
}

impl Drop for EngineReceiver {
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

struct Cancel {
    cancel_events: Option<oneshot::Sender<()>>,
}

impl Cancel {
    fn new(s: oneshot::Sender<()>) -> Self {
        Cancel {
            cancel_events: Some(s)
        }
    }
}

impl Drop for Cancel {
    fn drop(&mut self) {
        let cancel = mem::replace(&mut self.cancel_events, None);
        let _result = cancel.unwrap().send(());
    }
}

fn with_cancellation<T, U, R, E>(stream: T, cancellation: U) -> EventStream<R, E>
    where T: Stream<Item=R, Error=E> + 'static + Send,
          U: Future<Item=(), Error=E> + 'static + Send,
          E: Send + 'static,
          R: Send
{
    let result = stream.merge(cancellation.into_stream())
        .take_while(|e| {
            // stream continues as long as not canceled
            if let MergedItem::First(_) = *e {
                future::ok(true)
            } else {
                future::ok(false)
            }
        })
        .filter_map(|e| {
            match e {
                MergedItem::First(x) => Some(x),
                MergedItem::Second(_) => None,
                MergedItem::Both(x, _) => Some(x),
            }
        });

    Box::new(result)
}
