use super::iunknown::*;
use super::isrcentral::*;
use super::types::*;
use super::comptr::ComPtr;
use bstr::BStr;
use std::sync::Mutex;
use std::boxed::Box;

pub fn make_engine_sink(engine: ComPtr<IDgnSREngineControl>) -> RawComPtr {
    let obj = Box::into_raw(Box::new(EngineSink::new(engine)));

    obj as RawComPtr
}

#[repr(C)]
pub struct EngineSink {
    vtable1: *const ISRNotifySinkVtable,
    vtable2: *const IDgnGetSinkFlagsVtable,
    vtable3: *const IDgnSREngineNotifySinkVtable,
    ref_count: Mutex<u32>,
    engine: ComPtr<IDgnSREngineControl>,
}

#[allow(overflowing_literals)]
const E_NOINTERFACE: HRESULT = HRESULT(0x80004002 as i32);

#[allow(overflowing_literals)]
const E_POINTER: HRESULT = HRESULT(0x80004003 as i32);

impl EngineSink {
    fn new(engine: ComPtr<IDgnSREngineControl>) -> Self {
        EngineSink {
            vtable1: &v1::VTABLE,
            vtable2: &v2::VTABLE,
            vtable3: &v3::VTABLE,
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
    
    unsafe fn paused(&self, cookie: u64) -> HRESULT {
        println!("pause {}", cookie);
        let result = self.engine.resume(cookie);
        assert!(result.0 == 0);
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
