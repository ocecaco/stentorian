use super::iunknown::*;
use super::isrcentral::*;
use super::types::*;
use std::sync::Mutex;
use std::boxed::Box;
use libc::c_void;

pub fn make_grammar_sink() -> RawComPtr {
    let obj = Box::into_raw(Box::new(GrammarSink::new()));

    obj as RawComPtr
}

#[repr(C)]
pub struct GrammarSink {
    vtable1: *const ISRGramNotifySinkVtable,
    vtable2: *const IDgnGetSinkFlagsVtable,
    ref_count: Mutex<u32>
}

impl GrammarSink {
    fn new() -> Self {
        GrammarSink {
            vtable1: &v1::VTABLE,
            vtable2: &v2::VTABLE,
            ref_count: Mutex::new(1u32)
        }
    }

    unsafe fn query_interface(&self, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
        if v.is_null() {
            return E_POINTER;
        }

        let iid = &*iid;

        println!("grammar IID: {}", iid);

        if *iid == IUnknown::iid() {
            *v = &self.vtable1 as *const _ as RawComPtr;
            self.add_ref();
        } else if *iid == ISRGramNotifySink::iid() {
            *v = &self.vtable1 as *const _ as RawComPtr;
            self.add_ref();
        } else if *iid == IDgnGetSinkFlags::iid() {
            *v = &self.vtable2 as *const _ as RawComPtr;
            self.add_ref();
        } else {
            println!("fail!");
            return E_NOINTERFACE;
        }

        println!("success!");
        HRESULT(0)
    }

    unsafe fn add_ref(&self) -> u32 {
        let mut guard = self.ref_count.lock().unwrap();
        *guard += 1;
        *guard
    }

    unsafe fn release(&self) -> u32 {
        let result = {
            let mut guard = self.ref_count.lock().unwrap();
            *guard += 1;
            *guard
        };

        if result == 0 {
            Box::from_raw(self as *const _ as *mut GrammarSink);
        }

        result
    }

    unsafe fn bookmark(&self, x: u32) -> HRESULT {
        println!("grammar line: {}", line!());
        HRESULT(0)
    }
    unsafe fn paused(&self) -> HRESULT {
        println!("grammar line: {}", line!());
        HRESULT(0)
    }
    unsafe fn phrase_finish(&self, a: u32, b: u64, c: u64, phrase: *const c_void, results: RawComPtr) -> HRESULT {
        println!("grammar line: {}", line!());
        HRESULT(0)
    }
    unsafe fn phrase_hypothesis(&self, a: u32, b: u64, c: u64, phrase: *const c_void, results: RawComPtr) -> HRESULT {
        println!("grammar line: {}", line!());
        HRESULT(0)
    }
    unsafe fn phrase_start(&self, a: u64) -> HRESULT {
        println!("grammar line: {}", line!());
        HRESULT(0)
    }
    unsafe fn reevaluate(&self, a: RawComPtr) -> HRESULT {
        println!("grammar line: {}", line!());
        HRESULT(0)
    }
    unsafe fn training(&self, a: u32) -> HRESULT {
        println!("grammar line: {}", line!());
        HRESULT(0)
    }
    unsafe fn unarchive(&self, a: RawComPtr) -> HRESULT {
        println!("grammar line: {}", line!());
        HRESULT(0)
    }

    unsafe fn sink_flags_get(&self, flags: *mut u32) -> HRESULT {
        println!("grammar flags");
        *flags = 0xf1ff;
        HRESULT(0)
    }
}

#[allow(overflowing_literals)]
const E_NOINTERFACE: HRESULT = HRESULT(0x80004002 as i32);

#[allow(overflowing_literals)]
const E_POINTER: HRESULT = HRESULT(0x80004003 as i32);

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
                fn phrase_finish(a: u32, b: u64, c: u64, phrase: *const c_void, results: RawComPtr) -> HRESULT;
                fn phrase_hypothesis(a: u32, b: u64, c: u64, phrase: *const c_void, results: RawComPtr) -> HRESULT;
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
