use interfaces::*;
use components::*;
use components::comptr::ComPtr;
use components::refcount::*;
use std::boxed::Box;
use super::GrammarEvent;
use super::grammar_flags::GrammarSinkFlags;

use std::os::raw::c_void;

fn _ensure_kinds() {
    fn ensure_sync<T: Sync>() {}
    ensure_sync::<GrammarSink>();
}

pub struct Recognition {
    pub foreign: bool,
    pub ptr: ComPtr<IUnknown>,
}

pub type RawGrammarEvent = GrammarEvent<Recognition>;
pub type Callback = Box<Fn(RawGrammarEvent) + Sync>;

#[repr(C)]
pub struct GrammarSink {
    vtable1: &'static ISRGramNotifySinkVtable,
    vtable2: &'static IDgnGetSinkFlagsVtable,
    ref_count: RefCount,
    flags: GrammarSinkFlags,
    callback: Callback,
}

impl GrammarSink {
    pub fn create(flags: GrammarSinkFlags, callback: Callback) -> ComPtr<IUnknown> {
        let result = GrammarSink {
            vtable1: &v1::VTABLE,
            vtable2: &v2::VTABLE,
            ref_count: RefCount::new(1),
            flags: flags,
            callback: callback,
        };

        let raw = Box::into_raw(Box::new(result)) as RawComPtr;
        unsafe { raw_to_comptr(raw, true) }
    }

    fn send(&self, event: RawGrammarEvent) {
        (self.callback)(event);
    }

    unsafe fn query_interface(&self, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
        query_interface! {
            self, iid, v,
            IUnknown => vtable1,
            ISRGramNotifySink => vtable1,
            IDgnGetSinkFlags => vtable2
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
            Box::from_raw(self as *const _ as *mut GrammarSink);
        }

        result
    }

    fn bookmark(&self, x: u32) -> HRESULT {
        debug!("grammar event: bookmark {}", x);
        HRESULT(0)
    }
    fn paused(&self) -> HRESULT {
        debug!("grammar event: paused");
        HRESULT(0)
    }

    unsafe fn phrase_finish(&self,
                            flags: u32,
                            b: u64,
                            c: u64,
                            _phrase: *const c_void,
                            results: RawComPtr)
                            -> HRESULT {
        debug!("grammar event: phrase_finish {} {} {}", flags, b, c);

        const RECOGNIZED: u32 = 0x1;
        const THIS_GRAMMAR: u32 = 0x2;

        let reject = (flags & RECOGNIZED) == 0;
        let foreign = (flags & THIS_GRAMMAR) == 0;

        if reject {
            let event = GrammarEvent::PhraseRecognitionFailure;
            self.send(event);
        } else {
            let results = raw_to_comptr::<IUnknown>(results, false);
            let recognition = Recognition {
                foreign: foreign,
                ptr: results,
            };
            let event = GrammarEvent::PhraseFinish(recognition);
            self.send(event);
        }

        HRESULT(0)
    }

    fn phrase_hypothesis(&self,
                         flags: u32,
                         b: u64,
                         c: u64,
                         _phrase: *const c_void,
                         _results: RawComPtr)
                         -> HRESULT {
        debug!("grammar event: phrase_hypothesis {} {} {}", flags, b, c);
        HRESULT(0)
    }

    fn phrase_start(&self, a: u64) -> HRESULT {
        debug!("grammar event: phrase_start {}", a);
        self.send(GrammarEvent::PhraseStart);
        HRESULT(0)
    }

    fn reevaluate(&self, _a: RawComPtr) -> HRESULT {
        debug!("grammar event: reevaluate");
        HRESULT(0)
    }

    fn training(&self, a: u32) -> HRESULT {
        debug!("grammar event: training {}", a);
        HRESULT(0)
    }

    fn unarchive(&self, _a: RawComPtr) -> HRESULT {
        debug!("grammar event: unarchive");
        HRESULT(0)
    }

    unsafe fn sink_flags_get(&self, flags: *mut u32) -> HRESULT {
        debug!("grammar event: sink_flags_get");
        *flags = self.flags.bits();
        HRESULT(0)
    }
}


coclass! {
    GrammarSink {
        mod v1 in vtable1 {
            vtable_name: VTABLE,

            interface ISRGramNotifySink {
                vtable: ISRGramNotifySinkVtable,
                interface IUnknown {
                    vtable: IUnknownVtable,
                    fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                    fn add_ref() -> ULONG;
                    fn release() -> ULONG;
                },
                fn bookmark(x: u32) -> HRESULT;
                fn paused() -> HRESULT;
                fn phrase_finish(a: u32,
                                 b: u64,
                                 c: u64,
                                 phrase: *const c_void,
                                 results: RawComPtr) -> HRESULT;
                fn phrase_hypothesis(a: u32,
                                     b: u64,
                                     c: u64,
                                     phrase: *const c_void,
                                     results: RawComPtr) -> HRESULT;
                fn phrase_start(a: u64) -> HRESULT;
                fn reevaluate(a: RawComPtr) -> HRESULT;
                fn training(a: u32) -> HRESULT;
                fn unarchive(a: RawComPtr) -> HRESULT;
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
    }
}
