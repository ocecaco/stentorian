use dragon::*;
use components::*;
use components::bstr::BStr;

define_guid!(IID_ISRCentral = 0xB9BD3860,
             0x44DB,
             0x101B,
             0x90,
             0xA8,
             0x00,
             0xAA,
             0x00,
             0x3E,
             0x4B,
             0x50);

com_interface! {
    interface ISRCentral : IUnknown {
        iid: IID_ISRCentral,
        vtable: ISRCentralVtable,
        fn mode_get(info: *mut SRMODEINFO) -> HRESULT;
        fn grammar_load(fmt: SRGRMFMT,
                        data: SDATA,
                        sink: RawComPtr,
                        iid: IID,
                        control: *mut RawComPtr) -> HRESULT;
        fn pause() -> HRESULT;
        fn posn_get(posn: *mut u64) -> HRESULT;
        fn resume() -> HRESULT;
        fn to_filetime_todo() -> HRESULT;
        fn register(sink: RawComPtr, iid: IID, key: *mut u32) -> HRESULT;
        fn unregister(key: u32) -> HRESULT;
    }
}

define_guid!(IID_ISRNotifySink = 0x090CD9B0,
             0xDA1A,
             0x11CD,
             0xB3,
             0xCA,
             0x00,
             0xAA,
             0x00,
             0x47,
             0xBA,
             0x4F);

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

define_guid!(IID_IDgnSREngineNotifySink = 0xdd109001,
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
    interface IDgnSREngineNotifySink : IUnknown {
        iid: IID_IDgnSREngineNotifySink,
        vtable: IDgnSREngineNotifySinkVtable,
        fn attrib_changed_2(x: u32) -> HRESULT;
        fn paused(x: u64) -> HRESULT;
        fn mimic_done(x: u32, p: RawComPtr) -> HRESULT;
        fn error_happened(p: RawComPtr) -> HRESULT;
        fn progress(x: u32, s: BStr) -> HRESULT;
    }
}

define_guid!(IID_IDgnSREngineControl = 0xdd109000,
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
    interface IDgnSREngineControl : IUnknown {
        iid: IID_IDgnSREngineControl,
        vtable: IDgnSREngineControlVtable,
        fn get_version(a: *mut u16, b: *mut u16, c: *mut u16) -> HRESULT;
        fn get_mic_state(state: *mut u16) -> HRESULT;
        fn set_mic_state(state: u16, flag: i32) -> HRESULT;
        fn save_speaker(flag: i32) -> HRESULT;
        fn get_changed_info(flag: *mut i32, info: *mut u32) -> HRESULT;
        fn resume(cookie: u64) -> HRESULT;
    }
}

define_guid!(IID_ISRSpeaker = 0x090CD9AE,
             0xDA1A, 0x11CD, 0xB3, 0xCA, 0x0, 0xAA, 0x0, 0x47, 0xBA, 0x4F);

com_interface! {
    interface ISRSpeaker : IUnknown {
        iid: IID_ISRSpeaker,
        vtable: ISRSpeakerVtable,
        fn delete(name: BStr) -> HRESULT;
        fn enumerate(name: *mut *const u16, size: *mut u32) -> HRESULT;
        fn merge(name: BStr, a: *const (), b: u32) -> HRESULT;
        fn new(name: BStr) -> HRESULT;
        fn query(name: *mut u16, buffer_size: u32, required: *mut u32) -> HRESULT;
        fn read(name: BStr) -> HRESULT;
        fn revert(name: BStr) -> HRESULT;
        fn select(name: BStr, lock: i32) -> HRESULT;
        fn write(name: BStr, a: *const (), b: u32) -> HRESULT;
    }
}
