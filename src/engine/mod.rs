use std::ops::Deref;
use std::sync::mpsc::Receiver;
use components::comptr::ComPtr;
use components::*;
use interfaces::*;
use self::interfaces::*;
use std::ptr;
use dragon::*;
use self::event::*;
use self::enginesink::*;

mod interfaces;
mod enginesink;
mod event;

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

    pub fn register(&self, flags: EngineSinkFlags) -> EngineEventReceiver {
        let (sink, rx) = EngineSink::new(flags);
        let mut key = 0;
        unsafe {
            let result = self.central.register(sink,
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
