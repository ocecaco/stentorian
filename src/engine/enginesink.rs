use std::sync::mpsc::Sender;
use components::*;
use components::comptr::*;
use components::refcount::*;
use components::bstr::BStr;
use super::interfaces::*;
use interfaces::*;
use super::{EngineEvent, EngineSinkFlags, Attribute};
use std::boxed::Box;
use super::events::{EventSender, ConvertSender};

#[repr(C)]
pub struct EngineSink {
    vtable1: &'static ISRNotifySinkVtable,
    vtable2: &'static IDgnGetSinkFlagsVtable,
    vtable3: &'static IDgnSREngineNotifySinkVtable,
    ref_count: RefCount,
    flags: EngineSinkFlags,
    events: Box<EventSender<EngineEvent> + Sync>,
}

impl EngineSink {
    pub fn create<T>(flags: EngineSinkFlags, sender: Sender<T>) -> ComPtr<IUnknown>
        where T: From<EngineEvent> + Send + 'static
    {
        fn ensure_sync<T: Sync>(_: &T) {}

        let sink = EngineSink {
            vtable1: &v1::VTABLE,
            vtable2: &v2::VTABLE,
            vtable3: &v3::VTABLE,
            ref_count: RefCount::new(1),
            flags: flags,
            events: Box::new(ConvertSender::new(sender)),
        };

        ensure_sync(&sink);

        let raw = Box::into_raw(Box::new(sink)) as RawComPtr;
        unsafe { raw_to_comptr(raw, true) }
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
        self.events.send(EngineEvent::AttributeChanged(convert_attribute(a)));
        HRESULT(0)
    }

    fn interference(&self, a: u64, b: u64, c: u64) -> HRESULT {
        self.events.send(EngineEvent::Interference);
        HRESULT(0)
    }

    fn sound(&self, a: u64, b: u64) -> HRESULT {
        self.events.send(EngineEvent::Sound);
        HRESULT(0)
    }

    fn utterance_begin(&self, a: u64) -> HRESULT {
        self.events.send(EngineEvent::UtteranceBegin);
        HRESULT(0)
    }

    fn utterance_end(&self, a: u64, b: u64) -> HRESULT {
        self.events.send(EngineEvent::UtteranceEnd);
        HRESULT(0)
    }

    fn vu_meter(&self, a: u64, b: u16) -> HRESULT {
        self.events.send(EngineEvent::VuMeter);
        HRESULT(0)
    }


    unsafe fn sink_flags_get(&self, flags: *mut u32) -> HRESULT {
        *flags = self.flags.bits();
        HRESULT(0)
    }


    fn attrib_changed_2(&self, a: u32) -> HRESULT {
        self.events.send(EngineEvent::AttributeChanged(convert_attribute(a)));
        HRESULT(0)
    }

    unsafe fn paused(&self, cookie: u64) -> HRESULT {
        self.events
            .send(EngineEvent::Paused(PauseCookie(cookie)));
        HRESULT(0)
    }

    fn mimic_done(&self, x: u32, p: RawComPtr) -> HRESULT {
        self.events.send(EngineEvent::MimicDone);
        HRESULT(0)
    }

    fn error_happened(&self, p: RawComPtr) -> HRESULT {
        self.events.send(EngineEvent::ErrorHappened);
        HRESULT(0)
    }

    fn progress(&self, x: u32, s: BStr) -> HRESULT {
        self.events.send(EngineEvent::Progress);
        HRESULT(0)
    }
}

fn convert_attribute(a: u32) -> Attribute {
    match a {
        1 => Attribute::AutoGainEnable,
        2 => Attribute::Threshold,
        3 => Attribute::Echo,
        4 => Attribute::EnergyFloor,
        5 => Attribute::Microphone,
        6 => Attribute::RealTime,
        7 => Attribute::Speaker,
        8 => Attribute::Timeout,
        9 => Attribute::StartListening,
        10 => Attribute::StopListening,

        1001 => Attribute::MicrophoneState,
        1002 => Attribute::Registry,
        1003 => Attribute::PlaybackDone,
        1004 => Attribute::Topic,
        1005 => Attribute::LexiconAdd,
        1006 => Attribute::LexiconRemove,

        x => Attribute::Unknown(x)
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
