use super::EngineEvent;
use super::engine_flags::EngineSinkFlags;
use components::{raw_to_comptr, ComInterface, IUnknown, IUnknownVtable, RawComPtr, HRESULT, IID,
                 ULONG};
use components::bstr::BStr;
use components::comptr::ComPtr;
use components::refcount::RefCount;
use interfaces::{IDgnGetSinkFlags, IDgnGetSinkFlagsVtable, IDgnSREngineNotifySink,
                 IDgnSREngineNotifySinkVtable, ISRNotifySink, ISRNotifySinkVtable};
use std::boxed::Box;

fn _ensure_kinds() {
    fn ensure_sync<T: Sync>() {}
    ensure_sync::<EngineSink>();
}

pub type Callback = Box<Fn(EngineEvent) + Sync>;

#[repr(C)]
pub struct EngineSink {
    vtable1: &'static ISRNotifySinkVtable,
    vtable2: &'static IDgnGetSinkFlagsVtable,
    vtable3: &'static IDgnSREngineNotifySinkVtable,
    ref_count: RefCount,
    flags: EngineSinkFlags,
    callback: Callback,
}

impl EngineSink {
    pub fn create(flags: EngineSinkFlags, callback: Callback) -> ComPtr<IUnknown> {
        let sink = EngineSink {
            vtable1: &v1::VTABLE,
            vtable2: &v2::VTABLE,
            vtable3: &v3::VTABLE,
            ref_count: RefCount::new(1),
            flags: flags,
            callback: callback,
        };

        let raw = Box::into_raw(Box::new(sink)) as RawComPtr;
        unsafe { raw_to_comptr(raw, true) }
    }

    fn send(&self, event: EngineEvent) {
        (self.callback)(event);
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
        debug!("engine event: attrib_changed {}", a);
        if let Some(attr) = convert_attribute(a) {
            self.send(attr);
        }
        HRESULT(0)
    }

    fn interference(&self, a: u64, b: u64, c: u64) -> HRESULT {
        debug!("engine event: interference {} {} {}", a, b, c);
        HRESULT(0)
    }

    fn sound(&self, a: u64, b: u64) -> HRESULT {
        debug!("engine event: sound {} {}", a, b);
        HRESULT(0)
    }

    fn utterance_begin(&self, a: u64) -> HRESULT {
        debug!("engine event: utterance_begin {}", a);
        HRESULT(0)
    }

    fn utterance_end(&self, a: u64, b: u64) -> HRESULT {
        debug!("engine event: utterance_end {} {}", a, b);
        HRESULT(0)
    }

    fn vu_meter(&self, a: u64, b: u16) -> HRESULT {
        debug!("engine event: vu_meter {} {}", a, b);
        HRESULT(0)
    }

    unsafe fn sink_flags_get(&self, flags: *mut u32) -> HRESULT {
        debug!("engine event: sink_flags_get");
        *flags = self.flags.bits();
        HRESULT(0)
    }

    fn attrib_changed_2(&self, a: u32) -> HRESULT {
        debug!("engine event: attrib_changed_2 {}", a);
        if let Some(attr) = convert_attribute(a) {
            self.send(attr);
        }
        HRESULT(0)
    }

    unsafe fn paused(&self, cookie: u64) -> HRESULT {
        debug!("engine event: paused {}", cookie);
        self.send(EngineEvent::Paused(PauseCookie(cookie)));
        HRESULT(0)
    }

    fn mimic_done(&self, x: u32, _p: RawComPtr) -> HRESULT {
        debug!("engine event: mimic_done {}", x);
        HRESULT(0)
    }

    fn error_happened(&self, _p: RawComPtr) -> HRESULT {
        debug!("engine event: error_happened");
        HRESULT(0)
    }

    fn progress(&self, x: u32, _s: BStr) -> HRESULT {
        debug!("engine event: progress {}", x);
        HRESULT(0)
    }
}

fn convert_attribute(a: u32) -> Option<EngineEvent> {
    if a == 1001 {
        Some(EngineEvent::MicrophoneState)
    } else if a == 7 {
        Some(EngineEvent::UserChanged)
    } else {
        None
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
