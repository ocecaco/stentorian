#![allow(dead_code)]
extern crate libc;

#[macro_use(bitflags)]
extern crate bitflags;

#[macro_use]
mod macros;

#[allow(non_camel_case_types)]
mod types {
    use libc::c_void;

    bitflags! {
        #[repr(C)]
        pub flags COINIT: u32 {
            const COINIT_APARTMENTTHREADED  = 0x2,
            const COINIT_MULTITHREADED      = 0x0,
            const COINIT_DISABLE_OLE1DDE    = 0x4,
            const COINIT_SPEED_OVER_MEMORY  = 0x8
        }
    }

    bitflags! {
        #[repr(C)]
        pub flags CLSCTX: u32 {
            const CLSCTX_INPROC_SERVER           = 0x1,
            const CLSCTX_INPROC_HANDLER          = 0x2,
            const CLSCTX_LOCAL_SERVER            = 0x4,
            const CLSCTX_INPROC_SERVER16         = 0x8,
            const CLSCTX_REMOTE_SERVER           = 0x10,
            const CLSCTX_INPROC_HANDLER16        = 0x20,
            const CLSCTX_RESERVED1               = 0x40,
            const CLSCTX_RESERVED2               = 0x80,
            const CLSCTX_RESERVED3               = 0x100,
            const CLSCTX_RESERVED4               = 0x200,
            const CLSCTX_NO_CODE_DOWNLOAD        = 0x400,
            const CLSCTX_RESERVED5               = 0x800,
            const CLSCTX_NO_CUSTOM_MARSHAL       = 0x1000,
            const CLSCTX_ENABLE_CODE_DOWNLOAD    = 0x2000,
            const CLSCTX_NO_FAILURE_LOG          = 0x4000,
            const CLSCTX_DISABLE_AAA             = 0x8000,
            const CLSCTX_ENABLE_AAA              = 0x10000,
            const CLSCTX_FROM_DEFAULT_CONTEXT    = 0x20000,
            const CLSCTX_ACTIVATE_32_BIT_SERVER  = 0x40000,
            const CLSCTX_ACTIVATE_64_BIT_SERVER  = 0x80000,
            const CLSCTX_ENABLE_CLOAKING         = 0x100000,
            const CLSCTX_APPCONTAINER            = 0x400000,
            const CLSCTX_ACTIVATE_AAA_AS_IU      = 0x800000,
            const CLSCTX_PS_DLL                  = 0x80000000
        }
    }

    #[must_use]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[repr(C)]
    pub struct HRESULT(pub i32);

    pub type ULONG = u32;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    #[repr(C)]
    pub struct GUID {
        pub data1: u32,
        pub data2: u16,
        pub data3: u16,
        pub data4: [u8; 8]
    }

    pub type IID = GUID;
    pub type CLSID = GUID;

    pub type RawComPtr = *const c_void;
}

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

    define_guid!(pub CLSID_DgnDictate = 0xdd100001, 0x6205, 0x11cf, 0xae, 0x61, 0x00, 0x00, 0xe8, 0xa2, 0x86, 0x47);
    define_guid!(pub CLSID_DgnSite = 0xdd100006, 0x6205, 0x11cf, 0xae, 0x61, 0x00, 0x00, 0xe8, 0xa2, 0x86, 0x47);
}

mod iunknown {
    use super::types::*;

    define_guid!(IID_IUnknown = 0x00000000, 0x0000, 0x0000, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46);

    #[repr(C)]
    pub struct IUnknown {
        vtable: *const IUnknownVtable
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    pub struct IUnknownVtable {
        QueryInterface: extern "stdcall" fn(*const IUnknown, *const IID, *mut RawComPtr) -> HRESULT,
        AddRef: extern "stdcall" fn(*const IUnknown) -> ULONG,
        Release: extern "stdcall" fn(*const IUnknown) -> ULONG
    }

    #[allow(non_snake_case)]
    impl IUnknown {
        pub unsafe fn QueryInterface(&self, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
            ((*self.vtable).QueryInterface)(self, iid, v)
        }

        pub unsafe fn AddRef(&self) -> ULONG {
            ((*self.vtable).AddRef)(self)
        }

        pub unsafe fn Release(&self) -> ULONG {
            ((*self.vtable).Release)(self)
        }
    }

    unsafe impl ComInterface for IUnknown {
        type Vtable = IUnknownVtable;

        fn iid() -> IID {
            IID_IUnknown
        }
    }

    impl AsRef<IUnknown> for IUnknown {
        fn as_ref(&self) -> &IUnknown {
            self
        }
    }

    // unsafe to implement because it implies the type can safely be cast to IUnknown
    pub unsafe trait ComInterface: AsRef<IUnknown> {
        type Vtable;

        fn iid() -> IID;
    }
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

    define_guid!(IID_ISRCentral = 0xB9BD3860, 0x44DB, 0x101B, 0x90, 0xA8, 0x00, 0xAA, 0x00, 0x3E, 0x4B, 0x50);

    com_interface! {
        interface ISRCentral : IUnknown {
            iid: IID_ISRCentral,
            vtable: ISRCentralVtable,
            fn mode_get(info: *mut SRMODEINFO) -> HRESULT;
        }
    }
}

mod comptr {
    use super::iunknown::*;
    use std::ops::Deref;
    use std::ptr;

    pub struct ComPtr<T: ComInterface> {
        instance: *const T
    }

    impl<T: ComInterface> ComPtr<T> {
        pub unsafe fn from_raw(instance: *const T) -> ComPtr<T> {
            // TODO: check if pointer is null
            ComPtr { instance: instance }
        }
    }

    impl<T: ComInterface> Drop for ComPtr<T> {
        fn drop(&mut self) {
            let temp = self.instance;
            if !self.instance.is_null() {
                self.instance = ptr::null();
                unsafe {
                    let unk = (&*temp).as_ref();
                    unk.Release();
                }
            }
        }
    }

    impl<T: ComInterface> Deref for ComPtr<T> {
        type Target = T;

        fn deref(&self) -> &T {
            unsafe { &*self.instance }
        }
    }

    impl<T: ComInterface> Clone for ComPtr<T> {
        fn clone(&self) -> Self {
            let unk = self.as_ref();
            unsafe  {
                unk.AddRef();
            }

            ComPtr { instance: self.instance }
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
