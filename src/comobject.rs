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
    pub vtable: *const ISRGramNotifySinkVtable,
    ref_count: Mutex<Cell<u32>>
}

impl SpeechSink {
    fn new() -> Self {
        let unk_vtable = IUnknownVtable {
            query_interface: query_interface,
            add_ref: add_ref,
            release: release
        };

        let vtable = ISRGramNotifySinkVtable {
            base: unk_vtable,
            bookmark: bookmark,
            paused: paused,
            phrase_finish: phrase_finish,
            phrase_hypothesis: phrase_hypothesis,
            phrase_start: phrase_start,
            reevaluate: reevaluate,
            training: training,
            unarchive: unarchive
        };

        SpeechSink {
            vtable: Box::into_raw(Box::new(vtable)),
            ref_count: Mutex::new(Cell::new(1u32))
        }
    }
}

impl Drop for SpeechSink {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.vtable as *mut ISRGramNotifySinkVtable) };
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
    } else if *iid == ISRGramNotifySink::iid() {
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

extern "stdcall" fn bookmark(iface: *const ISRGramNotifySink, x: u32) -> HRESULT {
    println!("phrase_finish");
    HRESULT(0)
}

extern "stdcall" fn paused(iface: *const ISRGramNotifySink) -> HRESULT {
    println!("phrase_finish");
    HRESULT(0)
}

extern "stdcall" fn phrase_finish(iface: *const ISRGramNotifySink, a: u32, b: u64, c: u64, phrase: *const c_void, results: RawComPtr) -> HRESULT {
    println!("phrase_finish");
    HRESULT(0)
}

extern "stdcall" fn phrase_hypothesis(iface: *const ISRGramNotifySink, a: u32, b: u64, c: u64, phrase: *const c_void, results: RawComPtr) -> HRESULT {
    println!("phrase_finish");
    HRESULT(0)
}

extern "stdcall" fn phrase_start(iface: *const ISRGramNotifySink, a: u64) -> HRESULT {
    println!("phrase_finish");
    HRESULT(0)
}

extern "stdcall" fn reevaluate(iface: *const ISRGramNotifySink, a: RawComPtr) -> HRESULT {
    println!("phrase_finish");
    HRESULT(0)
}

extern "stdcall" fn training(iface: *const ISRGramNotifySink, a: u32) -> HRESULT {
    println!("phrase_finish");
    HRESULT(0)
}

extern "stdcall" fn unarchive(iface: *const ISRGramNotifySink, a: RawComPtr) -> HRESULT {
    println!("phrase_finish");
    HRESULT(0)
}
