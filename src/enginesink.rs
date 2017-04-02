use super::iunknown::*;
use super::isrcentral::*;
use super::types::*;
use super::comptr::ComPtr;
use bstr::BStr;
use std::sync::Mutex;
use std::boxed::Box;

pub fn make_engine_sink(engine: ComPtr<ISRCentral>) -> RawComPtr {
    let obj = Box::into_raw(Box::new(EngineSink::new(engine)));

    obj as RawComPtr
}

#[repr(C)]
pub struct EngineSink {
    vtable1: *const ISRNotifySinkVtable,
    vtable2: *const IDgnGetSinkFlagsVtable,
    vtable3: *const IDgnSREngineNotifySinkVtable,
    ref_count: Mutex<u32>,
    engine: ComPtr<ISRCentral>
}

#[allow(overflowing_literals)]
const E_NOINTERFACE: HRESULT = HRESULT(0x80004002 as i32);

#[allow(overflowing_literals)]
const E_POINTER: HRESULT = HRESULT(0x80004003 as i32);

impl EngineSink {
    fn new(engine: ComPtr<ISRCentral>) -> Self {
        let unk_vtable1 = IUnknownVtable {
            query_interface: v1::query_interface,
            add_ref: v1::add_ref,
            release: v1::release
        };

        let unk_vtable2 = IUnknownVtable {
            query_interface: v2::query_interface,
            add_ref: v2::add_ref,
            release: v2::release
        };

        let unk_vtable3 = IUnknownVtable {
            query_interface: v3::query_interface,
            add_ref: v3::add_ref,
            release: v3::release
        };
        
        let vtable1 = ISRNotifySinkVtable {
            base: unk_vtable1,
            attrib_changed: v1::attrib_changed,
            interference: v1::interference,
            sound: v1::sound,
            utterance_begin: v1::utterance_begin,
            utterance_end: v1::utterance_end,
            vu_meter: v1::vu_meter
        };

        let vtable2 = IDgnGetSinkFlagsVtable {
            base: unk_vtable2,
            sink_flags_get: v2::sink_flags_get
        };

        let vtable3 = IDgnSREngineNotifySinkVtable {
            base: unk_vtable3,
            attrib_changed_2: v3::attrib_changed_2,
            paused: v3::paused,
            mimic_done: v3::mimic_done,
            error_happened: v3::error_happened,
            progress: v3::progress
        };

        EngineSink {
            vtable1: Box::into_raw(Box::new(vtable1)),
            vtable2: Box::into_raw(Box::new(vtable2)),
            vtable3: Box::into_raw(Box::new(vtable3)),
            ref_count: Mutex::new(1u32),
            engine: engine
        }
    }

    unsafe fn query_interface(&self, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
        if v.is_null() {
            return E_POINTER;
        }

        let iid = &*iid;

        println!("engine IID: {}", iid);

        if *iid == IUnknown::iid() {
            *v = &self.vtable1 as *const _ as RawComPtr;
            self.add_ref();
        } else if *iid == ISRNotifySink::iid() {
            *v = &self.vtable1 as *const _ as RawComPtr;
            self.add_ref();
        } else if *iid == IDgnGetSinkFlags::iid() {
            *v = &self.vtable2 as *const _ as RawComPtr;
            self.add_ref();
        } else if *iid == IDgnSREngineNotifySink::iid() {
            *v = &self.vtable3 as *const _ as RawComPtr;
            self.add_ref();
        } else {
            println!("fail");
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
            Box::from_raw(self as *const _ as *mut EngineSink);
        }

        result
    }

    unsafe fn attrib_changed(&self, a: u32) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    
    unsafe fn interference(&self, a: u64, b: u64, c: u64) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    
    unsafe fn sound(&self, a: u64, b: u64) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    
    unsafe fn utterance_begin(&self, a: u64) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    
    unsafe fn utterance_end(&self, a: u64, b: u64) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    
    unsafe fn vu_meter(&self, a: u64, b: u16) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    

    unsafe fn sink_flags_get(&self, flags: *mut u32) -> HRESULT {
        println!("engine flags");
        *flags = 0x248;
        HRESULT(0)
    }
    

    unsafe fn attrib_changed_2(&self, x: u32) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    
    unsafe fn paused(&self, x: u64) -> HRESULT {
        println!("engine line: {}", line!());
        let result = self.engine.resume();
        assert_eq!(result.0, 0);
        HRESULT(0)
    }
    
    unsafe fn mimic_done(&self, x: u32, p: RawComPtr) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    
    unsafe fn error_happened(&self, p: RawComPtr) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
    
    unsafe fn progress(&self, x: u32, s: BStr) -> HRESULT {
        println!("engine line: {}", line!());
        HRESULT(0)
    }
}

com_stubs! {
    coclass EngineSink {
        mod v1 in vtable1 {
            interface IUnknown {
                fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                fn add_ref() -> ULONG;
                fn release() -> ULONG;
            }

            interface ISRNotifySink {
                fn attrib_changed(a: u32) -> HRESULT;
                fn interference(a: u64, b: u64, c: u64) -> HRESULT;
                fn sound(a: u64, b: u64) -> HRESULT;
                fn utterance_begin(a: u64) -> HRESULT;
                fn utterance_end(a: u64, b: u64) -> HRESULT;
                fn vu_meter(a: u64, b: u16) -> HRESULT;
            }

        }

        mod v2 in vtable2 {
            interface IUnknown {
                fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                fn add_ref() -> ULONG;
                fn release() -> ULONG;
            }

            interface IDgnGetSinkFlags {
                fn sink_flags_get(flags: *mut u32) -> HRESULT;
            }
        }

        mod v3 in vtable3 {
            interface IUnknown {
                fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                fn add_ref() -> ULONG;
                fn release() -> ULONG;
            }

            interface IDgnSREngineNotifySink {
                fn attrib_changed_2(x: u32) -> HRESULT;
                fn paused(x: u64) -> HRESULT;
                fn mimic_done(x: u32, p: RawComPtr) -> HRESULT;
                fn error_happened(p: RawComPtr) -> HRESULT;
                fn progress(x: u32, s: BStr) -> HRESULT;
            }
        }
    }
}
