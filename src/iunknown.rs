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
