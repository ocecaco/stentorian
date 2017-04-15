use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::Mutex;
use components::*;
use components::comptr::*;
use components::refcount::*;
use components::bstr::BStr;
use super::interfaces::*;
use interfaces::*;
use super::{EngineEvent, EngineSinkFlags};
use std::boxed::Box;

#[repr(C)]
pub struct EngineSink {
    vtable1: &'static ISRNotifySinkVtable,
    vtable2: &'static IDgnGetSinkFlagsVtable,
    vtable3: &'static IDgnSREngineNotifySinkVtable,
    ref_count: RefCount,
    flags: EngineSinkFlags,
    events: Mutex<Option<Sender<EngineEvent>>>,
}

impl EngineSink {
    pub fn new(flags: EngineSinkFlags) -> (ComPtr<IUnknown>, Receiver<EngineEvent>) {
        fn ensure_sync<T: Sync>(_: &T) {
        }

        let (tx, rx) = mpsc::channel();

        let sink = EngineSink {
            vtable1: &v1::VTABLE,
            vtable2: &v2::VTABLE,
            vtable3: &v3::VTABLE,
            ref_count: RefCount::new(1),
            flags: flags,
            events: Mutex::new(Some(tx)),
        };

        ensure_sync(&sink);

        let raw = Box::into_raw(Box::new(sink)) as RawComPtr;
        let unk = unsafe { raw_to_comptr(raw, true) };
        (unk, rx)
    }

    fn send_event(&self, event: EngineEvent) {
        let mut events = self.events.lock().unwrap();

        let result = if let Some(ref e) = *events {
            Some(e.send(event))
        } else {
            None
        };

        if let Some(r) = result {
            if r.is_err() {
                *events = None;
            }
        }
    }

    unsafe fn query_interface(&self, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
        query_interface! {
            self, iid, v,
            IUnknown => vtable1,
            ISRNotifySink => vtable1,
            IDgnGetSinkFlags => vtable2,
            IDgnSREngineNotifySink => vtable3
        }

        self.ref_count.up();
        HRESULT(0)
    }

    unsafe fn add_ref(&self) -> u32 {
        self.ref_count.up()
    }

    unsafe fn release(&self) -> u32 {
        let result = self.ref_count.down();

        if result == 0 {
            Box::from_raw(self as *const _ as *mut EngineSink);
        }

        result
    }

    fn attrib_changed(&self, a: u32) -> HRESULT {
        self.send_event(EngineEvent::AttributeChanged);
        HRESULT(0)
    }

    fn interference(&self, a: u64, b: u64, c: u64) -> HRESULT {
        self.send_event(EngineEvent::Interference);
        HRESULT(0)
    }

    fn sound(&self, a: u64, b: u64) -> HRESULT {
        self.send_event(EngineEvent::Sound);
        HRESULT(0)
    }

    fn utterance_begin(&self, a: u64) -> HRESULT {
        self.send_event(EngineEvent::UtteranceBegin);
        HRESULT(0)
    }

    fn utterance_end(&self, a: u64, b: u64) -> HRESULT {
        self.send_event(EngineEvent::UtteranceEnd);
        HRESULT(0)
    }

    fn vu_meter(&self, a: u64, b: u16) -> HRESULT {
        self.send_event(EngineEvent::VuMeter);
        HRESULT(0)
    }


    unsafe fn sink_flags_get(&self, flags: *mut u32) -> HRESULT {
        *flags = self.flags.bits();
        HRESULT(0)
    }


    fn attrib_changed_2(&self, x: u32) -> HRESULT {
        self.send_event(EngineEvent::AttributeChanged);
        HRESULT(0)
    }

    unsafe fn paused(&self, cookie: u64) -> HRESULT {
        self.send_event(EngineEvent::Paused(PauseCookie(cookie)));
        HRESULT(0)
    }

    fn mimic_done(&self, x: u32, p: RawComPtr) -> HRESULT {
        self.send_event(EngineEvent::MimicDone);
        HRESULT(0)
    }

    fn error_happened(&self, p: RawComPtr) -> HRESULT {
        self.send_event(EngineEvent::ErrorHappened);
        HRESULT(0)
    }

    fn progress(&self, x: u32, s: BStr) -> HRESULT {
        self.send_event(EngineEvent::Progress);
        HRESULT(0)
    }
}

#[derive(Debug)]
pub struct PauseCookie(u64);

impl From<PauseCookie> for u64 {
    fn from(cookie: PauseCookie) -> u64 {
        cookie.0
    }
}

coclass! {
    EngineSink {
        mod v1 in vtable1 {
            vtable_name: VTABLE,

            interface ISRNotifySink {
                vtable: ISRNotifySinkVtable,
                interface IUnknown {
                    vtable: IUnknownVtable,
                    fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                    fn add_ref() -> ULONG;
                    fn release() -> ULONG;
                },
                fn attrib_changed(a: u32) -> HRESULT;
                fn interference(a: u64, b: u64, c: u64) -> HRESULT;
                fn sound(a: u64, b: u64) -> HRESULT;
                fn utterance_begin(a: u64) -> HRESULT;
                fn utterance_end(a: u64, b: u64) -> HRESULT;
                fn vu_meter(a: u64, b: u16) -> HRESULT;
            }
        }

        mod v2 in vtable2 {
            vtable_name: VTABLE,

            interface IDgnGetSinkFlags {
                vtable: IDgnGetSinkFlagsVtable,
                interface IUnknown {
                    vtable: IUnknownVtable,
                    fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                    fn add_ref() -> ULONG;
                    fn release() -> ULONG;
                },
                fn sink_flags_get(flags: *mut u32) -> HRESULT;
            }
        }

        mod v3 in vtable3 {
            vtable_name: VTABLE,

            interface IDgnSREngineNotifySink {
                vtable: IDgnSREngineNotifySinkVtable,
                interface IUnknown {
                    vtable: IUnknownVtable,
                    fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                    fn add_ref() -> ULONG;
                    fn release() -> ULONG;
                },
                fn attrib_changed_2(x: u32) -> HRESULT;
                fn paused(x: u64) -> HRESULT;
                fn mimic_done(x: u32, p: RawComPtr) -> HRESULT;
                fn error_happened(p: RawComPtr) -> HRESULT;
                fn progress(x: u32, s: BStr) -> HRESULT;
            }
        }
    }
}
