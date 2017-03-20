#![allow(unused_variables)]

use super::iunknown::*;
use super::isrcentral::*;
use super::types::*;
use std::sync::Mutex;
use std::cell::Cell;
use std::boxed::Box;
use libc::c_void;

pub fn make_test_object() -> RawComPtr {
    let obj = Box::into_raw(Box::new(SpeechSink::new()));

    obj as RawComPtr
}

#[repr(C)]
pub struct SpeechSink {
    // TODO: possibly store virtual table inside object itself to
    // prevent two heap allocations
    pub vtable: *const ISRNotifySinkVtable,
    ref_count: Mutex<Cell<u32>>
}

impl SpeechSink {
    fn new() -> Self {
        let unk_vtable = IUnknownVtable {
            query_interface: query_interface,
            add_ref: add_ref,
            release: release
        };

        let vtable = ISRNotifySinkVtable {
            base: unk_vtable,
            attrib_changed: attrib_changed,
            interference: interference,
            sound: sound,
            utterance_begin: utterance_begin,
            utterance_end: utterance_end,
            vu_meter: vu_meter
        };

        SpeechSink {
            vtable: Box::into_raw(Box::new(vtable)),
            ref_count: Mutex::new(Cell::new(1u32))
        }
    }
}

impl Drop for SpeechSink {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.vtable as *mut ISRNotifySinkVtable) };
    }
}

#[allow(overflowing_literals)]
const E_NOINTERFACE: HRESULT = HRESULT(0x80004002 as i32);

#[allow(overflowing_literals)]
const E_POINTER: HRESULT = HRESULT(0x80004003 as i32);

extern "stdcall" fn query_interface(iface: *const IUnknown, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
    println!("query_interface");
    if v.is_null() {
        return E_POINTER;
    }
    
    let iid = unsafe { &*iid };

    if *iid == IUnknown::iid() {
        unsafe { *v = iface as RawComPtr };
        add_ref(iface);
    } else if *iid == ISRNotifySink::iid() {
        unsafe { *v = iface as RawComPtr };
        add_ref(iface);
    } else {
        return E_NOINTERFACE;
    }
    
    HRESULT(0)
}

extern "stdcall" fn add_ref(iface: *const IUnknown) -> ULONG {
    println!("add_ref");
    let obj: &SpeechSink = unsafe { &*(iface as *const SpeechSink) };

    let new_value = {
        let guard = obj.ref_count.lock().unwrap();

        let old_value = guard.get();
        let new_value = old_value + 1;
        guard.set(new_value);

        new_value
    };

    new_value
}

extern "stdcall" fn release(iface: *const IUnknown) -> ULONG {
    println!("release");
    let obj: &SpeechSink = unsafe { &*(iface as *const SpeechSink) };

    let new_value = {
        let guard = obj.ref_count.lock().unwrap();

        let old_value = guard.get();
        let new_value = old_value - 1;
        guard.set(new_value);

        new_value
    };

    if new_value == 0 {
        let ptr = iface as *mut SpeechSink;
        unsafe { Box::from_raw(ptr) };
    }

    new_value
}

extern "stdcall" fn attrib_changed(ptr: *const ISRNotifySink, a: u32) -> HRESULT {
    println!("function called");
    HRESULT(0)
}

extern "stdcall" fn interference(ptr: *const ISRNotifySink, a: u64, b: u64, c: u64) -> HRESULT {
    println!("function called");
    HRESULT(0)
}

extern "stdcall" fn sound(ptr: *const ISRNotifySink, a: u64, b: u64) -> HRESULT {
    println!("function called");
    HRESULT(0)
}

extern "stdcall" fn utterance_begin(ptr: *const ISRNotifySink, a: u64) -> HRESULT {
    println!("function called");
    HRESULT(0)
}

extern "stdcall" fn utterance_end(ptr: *const ISRNotifySink, a: u64, b: u64) -> HRESULT {
    println!("function called");
    HRESULT(0)
}

extern "stdcall" fn vu_meter(ptr: *const ISRNotifySink, a: u64, b: u16) -> HRESULT {
    println!("function called");
    HRESULT(0)
}
