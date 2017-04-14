use std::ops::Deref;
use std::sync::mpsc::Receiver;
use components::comptr::ComPtr;
use components::*;
use interfaces::*;
use self::interfaces::*;
use std::ptr;
use dragon::*;
use self::enginesink::*;

mod interfaces;
mod enginesink;

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

pub struct Engine {
    central: ComPtr<ISRCentral>,
    engine_control: ComPtr<IDgnSREngineControl>,
}

pub struct EngineEventReceiver {
    central: ComPtr<ISRCentral>,
    register_key: u32,
    receiver: Receiver<EngineEvent>,
}

impl Deref for EngineEventReceiver {
    type Target = Receiver<EngineEvent>;

    fn deref(&self) -> &Receiver<EngineEvent> {
        &self.receiver
    }
}

impl Drop for EngineEventReceiver {
    fn drop(&mut self) {
        unsafe {
            let result = self.central.unregister(self.register_key);
            assert_eq!(result.0, 0);
        }
    }
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

    pub fn register(&self, flags: EngineSinkFlags) -> EngineEventReceiver {
        let (sink, rx) = EngineSink::new(flags);
        let mut key = 0;
        unsafe {
            let result = self.central.register(&sink as &IUnknown as *const _ as RawComPtr,
                                               IDgnSREngineNotifySink::iid(),
                                               &mut key);
            assert_eq!(result.0, 0);
        }

        EngineEventReceiver {
            central: self.central.clone(),
            register_key: key,
            receiver: rx,
        }
    }
}

fn get_central() -> ComPtr<ISRCentral> {
    let provider =
        create_instance::<IServiceProvider>(&CLSID_DgnSite, None, CLSCTX_LOCAL_SERVER).unwrap();
    unsafe {
        let mut central: RawComPtr = ptr::null();
        let result =
            provider.query_service(&CLSID_DgnDictate, &ISRCentral::iid(), &mut central);
        assert_eq!(result.0, 0);
        raw_to_comptr::<ISRCentral>(central, true)
    }
}
