use components::*;

define_guid!(IID_IServiceProvider = 0x6d5140c1,
             0x7436,
             0x11ce,
             0x80,
             0x34,
             0x00,
             0xaa,
             0x00,
             0x60,
             0x09,
             0xfa);

com_interface! {
    interface IServiceProvider : IUnknown {
        iid: IID_IServiceProvider,
        vtable: IServiceProviderVtable,
        fn query_service(guid: *const GUID, iid: *const IID, v: *mut RawComPtr) -> HRESULT;
    }
}

define_guid!(IID_IDgnGetSinkFlags = 0xdd108010,
             0x6205,
             0x11cf,
             0xae,
             0x61,
             0x00,
             0x00,
             0xe8,
             0xa2,
             0x86,
             0x47);

com_interface! {
    interface IDgnGetSinkFlags : IUnknown {
        iid: IID_IDgnGetSinkFlags,
        vtable: IDgnGetSinkFlagsVtable,
        fn sink_flags_get(flags: *mut u32) -> HRESULT;
    }
}
