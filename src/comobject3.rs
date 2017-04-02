use std::sync::Mutex;
use std::boxed::Box;

use super::iunknown::*;
use super::types::*;

macro_rules! offset_of {
    ($t:ty, $f:ident) => {
        unsafe { &(*(0 as *const $t)).$f as *const _ as usize }
    }
}

define_guid!(IID_IA = 0x6d5240c3, 0x7436, 0x11ce, 0x80, 0x34, 0x00, 0xaa, 0x00, 0x60, 0x09, 0xfa);
define_guid!(IID_IB = 0x6d5240c4, 0x7436, 0x11ce, 0x80, 0x34, 0x00, 0xaa, 0x00, 0x60, 0x09, 0xfa);

com_interface! {
    interface IA : IUnknown {
        iid: IID_IA,
        vtable: IAVtable,
        fn a(v: *mut u32) -> HRESULT;
    }
}

com_interface! {
    interface IB : IUnknown {
        iid: IID_IB,
        vtable: IBVtable,
        fn b(v: *mut u32) -> HRESULT;
    }
}

#[repr(C)]
pub struct Test {
    interface_a: *const IAVtable,
    interface_b: *const IBVtable,
    state: Mutex<TestState>
}

#[allow(overflowing_literals)]
const E_NOINTERFACE: HRESULT = HRESULT(0x80004002 as i32);

#[allow(overflowing_literals)]
const E_POINTER: HRESULT = HRESULT(0x80004003 as i32);

struct TestState {
    ref_count: u32
}

impl Drop for Test {
    fn drop(&mut self) {
        println!("destroying test object");
        unsafe {
            Box::from_raw(self.interface_a as *mut IAVtable);
            Box::from_raw(self.interface_b as *mut IBVtable);
        }
    }
}

impl Test {
    pub fn new() -> RawComPtr {
        println!("creating test object");
        let result = Box::into_raw(Box::new(Test {
            interface_a: Box::into_raw(Box::new(IAVtable {
                base: IUnknownVtable {
                    query_interface: query_interface_a,
                    add_ref: add_ref_a,
                    release: release_a
                },
                a: a
            })),
            interface_b: Box::into_raw(Box::new(IBVtable {
                base: IUnknownVtable {
                    query_interface: query_interface_b,
                    add_ref: add_ref_b,
                    release: release_b
                },
                b: b
            })),
            state: Mutex::new(TestState {
                ref_count: 1
            })
        }));

        result as RawComPtr
    }
    
    unsafe fn query_interface(&self, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
        println!("query interface");
        if v.is_null() {
            return E_POINTER;
        }

        let iid = &*iid;

        if *iid == IUnknown::iid() {
            *v = &self.interface_a as *const _ as RawComPtr;
            self.add_ref();
        } else if *iid == IA::iid() {
            *v = &self.interface_a as *const _ as RawComPtr;
            self.add_ref();
        } else if *iid == IB::iid() {
            *v = &self.interface_b as *const _ as RawComPtr;
            self.add_ref();
        } else {
            return E_NOINTERFACE;
        }

        HRESULT(0)
    }

    unsafe fn add_ref(&self) -> u32 {
        println!("add reference");
        let mut guard = self.state.lock().unwrap();
        guard.ref_count += 1;
        guard.ref_count
    }

    unsafe fn release(&self) -> u32 {
        println!("release");
        let result = {
            let mut guard = self.state.lock().unwrap();
            guard.ref_count -= 1;
            guard.ref_count
        };

        if result == 0 {
            Box::from_raw(self as *const _ as *mut Test);
        }

        result
    }
    
    unsafe fn a(&self, v: *mut u32) -> HRESULT {
        println!("a");
        *v = 3; 

        HRESULT(0)
    }

    unsafe fn b(&self, v: *mut u32) -> HRESULT {
        println!("b");
        *v = 5;

        HRESULT(0)
    }
}

extern "stdcall" fn query_interface_a(ptr: *const IUnknown, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
    let this = (ptr as usize - offset_of!(Test, interface_a)) as *const Test;
    let this = unsafe { &*this };

    unsafe { this.query_interface(iid, v) }
}

extern "stdcall" fn add_ref_a(ptr: *const IUnknown) -> u32 {
    let this = (ptr as usize - offset_of!(Test, interface_a)) as *const Test;
    let this = unsafe { &*this };

    unsafe { this.add_ref() }
}

extern "stdcall" fn release_a(ptr: *const IUnknown) -> u32 {
    let this = (ptr as usize - offset_of!(Test, interface_a)) as *const Test;
    let this = unsafe { &*this };

    unsafe { this.release() }
}

extern "stdcall" fn a(ptr: *const IA, v: *mut u32) -> HRESULT {
    let this = (ptr as usize - offset_of!(Test, interface_a)) as *const Test;
    let this = unsafe { &*this };

    unsafe { this.a(v) }
}

extern "stdcall" fn query_interface_b(ptr: *const IUnknown, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
    let this = (ptr as usize - offset_of!(Test, interface_b)) as *const Test;
    let this = unsafe { &*this };

    unsafe { this.query_interface(iid, v) }
}

extern "stdcall" fn add_ref_b(ptr: *const IUnknown) -> u32 {
    let this = (ptr as usize - offset_of!(Test, interface_b)) as *const Test;
    let this = unsafe { &*this };

    unsafe { this.add_ref() }
}

extern "stdcall" fn release_b(ptr: *const IUnknown) -> u32 {
    let this = (ptr as usize - offset_of!(Test, interface_b)) as *const Test;
    let this = unsafe { &*this };

    unsafe { this.release() }
}

extern "stdcall" fn b(ptr: *const IB, v: *mut u32) -> HRESULT {
    let this = (ptr as usize - offset_of!(Test, interface_b)) as *const Test;
    let this = unsafe { &*this };

    unsafe { this.b(v) }
}
