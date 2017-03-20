#![allow(unused_variables)]
#![allow(dead_code)]

extern crate libc;

#[macro_use(bitflags)]
extern crate bitflags;

extern crate winapi;
extern crate user32;

#[macro_use]
mod macros;

mod bstr;
mod comptr;
mod iunknown;
mod types;
mod comobject;
mod comobject2;

mod dragon {
    use super::types::*;

    type LANGID = u16;

    const LANG_LEN: usize = 64;
    const SVFN_LEN: usize = 262;
    const SRMI_NAMELEN: usize = SVFN_LEN;

    #[allow(non_snake_case)]
    #[repr(C)]
    pub struct LANGUAGE {
        LanguageID: LANGID,
        szDialect: [u16; LANG_LEN]
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    pub struct SRMODEINFO {
        gEngineID: GUID,
        szMfgName: [u16; SRMI_NAMELEN],
        pub szProductName: [u16; SRMI_NAMELEN],
        gModeID: GUID,
        szModeName: [u16; SRMI_NAMELEN],
        language: LANGUAGE,
        dwSequencing: u32,
        dwMaxWordsVocab: u32,
        dwMaxWordsState: u32,
        dwGrammars: u32,
        dwFeatures: u32,
        dwInterfaces: u32,
        dwEngineFeatures: u32
    }

    #[repr(C)]
    pub enum SRGRMFMT {
        SRGRMFMT_CFG = 0x0000,
        SRGRMFMT_LIMITEDDOMAIN = 0x0001,
        SRGRMFMT_DICTATION = 0x0002,
        SRGRMFMT_CFGNATIVE = 0x8000,
        SRGRMFMT_LIMITEDDOMAINNATIVE = 0x8001,
        SRGRMFMT_DICTATIONNATIVE = 0x8002,
        SRGRMFMT_DRAGONNATIVE1 = 0x8101,
        SRGRMFMT_DRAGONNATIVE2 = 0x8102,
        SRGRMFMT_DRAGONNATIVE3 = 0x8103
    }

    #[repr(C)]
    pub struct SDATA {
        pub data: *const u8,
        pub size: u32
    }

    define_guid!(pub CLSID_DgnDictate = 0xdd100001, 0x6205, 0x11cf, 0xae, 0x61, 0x00, 0x00, 0xe8, 0xa2, 0x86, 0x47);
    define_guid!(pub CLSID_DgnSite = 0xdd100006, 0x6205, 0x11cf, 0xae, 0x61, 0x00, 0x00, 0xe8, 0xa2, 0x86, 0x47);
}


mod iserviceprovider {
    use super::types::*;
    use super::iunknown::*;

    define_guid!(IID_IServiceProvider = 0x6d5140c1, 0x7436, 0x11ce, 0x80, 0x34, 0x00, 0xaa, 0x00, 0x60, 0x09, 0xfa);

    com_interface! {
        interface IServiceProvider : IUnknown {
            iid: IID_IServiceProvider,
            vtable: IServiceProviderVtable,
            fn query_service(guid: *const GUID, iid: *const IID, v: *mut RawComPtr) -> HRESULT;
        }
    }
}

mod isrcentral {
    use super::types::*;
    use super::iunknown::*;
    use super::dragon::*;
    use super::bstr::*;
    use libc::c_void;

    define_guid!(IID_ISRCentral = 0xB9BD3860, 0x44DB, 0x101B, 0x90, 0xA8, 0x00, 0xAA, 0x00, 0x3E, 0x4B, 0x50);

    com_interface! {
        interface ISRCentral : IUnknown {
            iid: IID_ISRCentral,
            vtable: ISRCentralVtable,
            fn mode_get(info: *mut SRMODEINFO) -> HRESULT;
            fn grammar_load(fmt: SRGRMFMT, data: SDATA, sink: RawComPtr, iid: IID, control: *mut RawComPtr) -> HRESULT;
            fn pause() -> HRESULT;
            fn posn_get(posn: *mut u64) -> HRESULT;
            fn resume() -> HRESULT;
            fn to_filetime_todo() -> HRESULT;
            fn register(sink: RawComPtr, iid: IID, key: *mut u32) -> HRESULT;
            fn unregister(key: u32) -> HRESULT;
        }
    }

    define_guid!(IID_ISRGramNotifySink = 0xf106bfa0, 0xc743, 0x11cd, 0x80, 0xe5, 0x0, 0xaa, 0x0, 0x3e, 0x4b, 0x50);
    
    com_interface! {
        interface ISRGramNotifySink : IUnknown {
            iid: IID_ISRGramNotifySink,
            vtable: ISRGramNotifySinkVtable,
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

    define_guid!(IID_ISRGramCommon = 0xe8c3e160, 0xc743, 0x11cd, 0x80, 0xe5, 0x0, 0xaa, 0x0, 0x3e, 0x4b, 0x50);

    com_interface! {
        interface ISRGramCommon : IUnknown {
            iid: IID_ISRGramCommon,
            vtable: ISRGramCommonVtable,
            fn activate(w: HWND, autopause: i32, rule_name: BStr) -> HRESULT;
        }
    }

    define_guid!(IID_ISRNotifySink = 0x090CD9B0, 0xDA1A, 0x11CD, 0xB3, 0xCA, 0x00, 0xAA, 0x00, 0x47, 0xBA, 0x4F);

    com_interface! {
        interface ISRNotifySink : IUnknown {
            iid: IID_ISRNotifySink,
            vtable: ISRNotifySinkVtable,
            fn attrib_changed(a: u32) -> HRESULT;
            fn interference(a: u64, b: u64, c: u64) -> HRESULT;
            fn sound(a: u64, b: u64) -> HRESULT;
            fn utterance_begin(a: u64) -> HRESULT;
            fn utterance_end(a: u64, b: u64) -> HRESULT;
            fn vu_meter(a: u64, b: u16) -> HRESULT;
        }
    }
}


mod api {
    use types::*;
    use libc::{c_void};
    use std::ptr;
    use std::mem;
    use std::{thread, time};
    use super::dragon::*;
    use super::iunknown::*;
    use super::iserviceprovider::*;
    use super::isrcentral::*;
    use super::comptr::*;
    use super::comobject::*;
    use super::comobject2;
    use super::bstr::*;

    use winapi::winuser::MSG;
    use user32::{GetMessageW, TranslateMessage, DispatchMessageW};

    use std::fs::File;
    use std::io::Read;

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(reserved: *const c_void, coinit: COINIT) -> HRESULT;
        fn CoUninitialize();

        fn CoCreateInstance(clsid: *const CLSID, unk_outer: RawComPtr, cls_context: CLSCTX, iid: *const IID, v: *mut RawComPtr) -> HRESULT;
    }

    unsafe fn raw_to_comptr<T: ComInterface>(ptr: RawComPtr) -> ComPtr<T> {
        let interface_ptr: *const T = ptr as *const T;
        ComPtr::from_raw(interface_ptr)
    }

    // TODO: Ensure initialization has been called
    fn create_instance<U>(clsid: &CLSID, unk_outer: Option<&IUnknown>, cls_context: CLSCTX) -> Option<ComPtr<U>> where U: ComInterface {
        let mut ptr: RawComPtr = ptr::null();
        let outer: *const IUnknown = if let Some(x) = unk_outer { x } else { ptr::null() };
        let result = unsafe { CoCreateInstance(clsid, outer as RawComPtr, cls_context, &U::iid(), &mut ptr) };

        if result.0 != 0 {
            None
        } else {
            unsafe { Some(raw_to_comptr(ptr)) }
        }
    }

    fn query_interface<U: ComInterface>(unk: &IUnknown) -> Option<ComPtr<U>> {
        let mut ptr: RawComPtr = ptr::null();

        let result = unsafe { unk.query_interface(&U::iid(), &mut ptr) };

        if result.0 != 0 {
            None
        } else {
            unsafe { Some(raw_to_comptr(ptr)) }
        }
    }

    pub fn test() {
        unsafe {
            let result: HRESULT = CoInitializeEx(ptr::null(), COINIT_APARTMENTTHREADED);
            assert_eq!(result.0, 0);
        }

        let mut file = File::open("C:\\Users\\Daniel\\Documents\\grammar_test.bin").unwrap();
        let mut grammar: Vec<u8> = Vec::new();
        file.read_to_end(&mut grammar).unwrap();

        if let Some(obj) = create_instance::<IServiceProvider>(&CLSID_DgnSite, None, CLSCTX_LOCAL_SERVER) {
            let obj2 = unsafe {
                let mut central: RawComPtr = ptr::null();
                let result = obj.query_service(&CLSID_DgnDictate, &ISRCentral::iid(), &mut central);
                assert_eq!(result.0, 0);
                raw_to_comptr::<ISRCentral>(central)
            };

            let mut info: SRMODEINFO = unsafe { mem::uninitialized() };
            unsafe {
                assert_eq!(obj2.mode_get(&mut info).0, 0);
            }

            println!("{}", String::from_utf16_lossy(&(&info.szProductName)
                                                    .iter()
                                                    .cloned()
                                                    .take_while(|&x| x != 0)
                                                    .collect::<Vec<u16>>()));

            let mut key = 0u32;
            let result = unsafe {
                obj2.register(comobject2::make_test_object(),
                              ISRNotifySink::iid(),
                              &mut key)
            };
            assert_eq!(result.0, 0);
            println!("{}", key);

            let grammar_slice: &[u8] = &grammar;
            let mut control: RawComPtr = ptr::null();
            let result = unsafe {
                obj2.grammar_load(SRGRMFMT::SRGRMFMT_CFG,
                                  SDATA {
                                      data: grammar_slice.as_ptr(),
                                      size: grammar_slice.len() as u32
                                  },
                                  make_test_object(),
                                  ISRGramNotifySink::iid(),
                                  &mut control)
            };
            assert_eq!(result.0, 0);

            println!("hello");

            let grammar_control = unsafe { raw_to_comptr::<IUnknown>(control) };
            let grammar_control = query_interface::<ISRGramCommon>(&grammar_control).unwrap();
            unsafe  {
                let result = grammar_control.activate(ptr::null(), 0, BString::from("Mapping").as_ref());
                assert_eq!(result.0 as u32, 0);
            }

            println!("hello");
        }

        let mut msg = unsafe { mem::uninitialized() };
        while unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } > 0 {
            println!("received message");
            unsafe {
                TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            }
        }
        
        unsafe {
            CoUninitialize();
        }
    }
}

fn main() {
    api::test();
}
