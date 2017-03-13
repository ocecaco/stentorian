#![allow(dead_code)]
extern crate libc;

#[macro_use(bitflags)]
extern crate bitflags;

#[macro_use]
mod macros;

mod comptr;
mod iunknown;
mod types;

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
        data: *const u8,
        size: u32
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
    use libc::c_void;

    define_guid!(IID_ISRCentral = 0xB9BD3860, 0x44DB, 0x101B, 0x90, 0xA8, 0x00, 0xAA, 0x00, 0x3E, 0x4B, 0x50);

    com_interface! {
        interface ISRCentral : IUnknown {
            iid: IID_ISRCentral,
            vtable: ISRCentralVtable,
            fn mode_get(info: *mut SRMODEINFO) -> HRESULT;
            fn grammar_load(fmt: SRGRMFMT, data: SDATA, sink: RawComPtr, iid: IID, control: *mut RawComPtr) -> HRESULT;
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
}


mod api {
    use types::*;
    use libc::{c_void};
    use std::ptr;
    use std::mem;
    use super::dragon::*;
    use super::iunknown::*;
    use super::iserviceprovider::*;
    use super::isrcentral::*;
    use super::comptr::*;

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

    pub fn test() {
        unsafe {
            let result: HRESULT = CoInitializeEx(ptr::null(), COINIT_APARTMENTTHREADED);
            assert_eq!(result.0, 0);
        }

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
        }

        unsafe {
            CoUninitialize();
        }
    }
}

fn main() {
    api::test();
}
