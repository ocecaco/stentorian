use components::*;
use components::bstr::*;
use dragon::*;
use std::os::raw::c_void;

define_guid!(IID_ISRGramNotifySink = 0xf106bfa0,
             0xc743,
             0x11cd,
             0x80,
             0xe5,
             0x0,
             0xaa,
             0x0,
             0x3e,
             0x4b,
             0x50);

com_interface! {
    interface ISRGramNotifySink : IUnknown {
        iid: IID_ISRGramNotifySink,
        vtable: ISRGramNotifySinkVtable,
        fn bookmark(x: u32) -> HRESULT;
        fn paused() -> HRESULT;
        fn phrase_finish(a: u32,
                         b: u64,
                         c: u64,
                         phrase: *const c_void,
                         results: RawComPtr) -> HRESULT;
        fn phrase_hypothesis(a: u32,
                             b: u64,
                             c: u64,
                             phrase: *const c_void,
                             results: RawComPtr) -> HRESULT;
        fn phrase_start(a: u64) -> HRESULT;
        fn reevaluate(a: RawComPtr) -> HRESULT;
        fn training(a: u32) -> HRESULT;
        fn unarchive(a: RawComPtr) -> HRESULT;
    }
}

define_guid!(IID_ISRGramCommon = 0xe8c3e160,
             0xc743,
             0x11cd,
             0x80,
             0xe5,
             0x0,
             0xaa,
             0x0,
             0x3e,
             0x4b,
             0x50);

com_interface! {
    interface ISRGramCommon : IUnknown {
        iid: IID_ISRGramCommon,
        vtable: ISRGramCommonVtable,
        fn activate(w: HWND, autopause: i32, rule_name: BStr) -> HRESULT;
        fn todo1() -> HRESULT;
        fn todo2() -> HRESULT;
        fn deactivate(rule_name: BStr) -> HRESULT;
    }
}

define_guid!(IID_IDgnSRGramCommon = 0xdd108006,
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
    interface IDgnSRGramCommon : IUnknown {
        iid: IID_IDgnSRGramCommon,
        vtable: IDgnSRGramCommonVtable,
        fn special_grammar(exclusive: i32) -> HRESULT;
        fn identify(g: *mut GUID) -> HRESULT;
    }
}

define_guid!(IID_ISRResGraph = 0x090CD9AA,
             0xDA1A,
             0x11CD,
             0xB3,
             0xCA,
             0x0,
             0xAA,
             0x0,
             0x47,
             0xBA,
             0x4F);

com_interface! {
    interface ISRResGraph : IUnknown {
        iid: IID_ISRResGraph,
        vtable: ISRResGraphVtable,
        fn best_path_phoneme(choice: u32,
                             path: *mut u32,
                             max_path_size: u32,
                             actual_path_size: *mut u32) -> HRESULT;
        fn best_path_word(choice: u32,
                          path: *mut u32,
                          max_path_size: u32,
                          actual_path_size: *mut u32) -> HRESULT;
        fn get_phoneme_node(idx: u32,
                            phoneme_node: *const c_void,
                            a: *const c_void,
                            b: *const c_void) -> HRESULT;
        fn get_word_node(idx: u32,
                         word_node: *mut SRRESWORDNODE,
                         word: *mut SRWORD,
                         max_word_size: u32,
                         size_needed: *mut u32) -> HRESULT;
    }
}

define_guid!(IID_ISRGramCFG = 0xecc0b180,
             0xc743,
             0x11cd,
             0x80,
             0xe5,
             0x0,
             0xaa,
             0x0,
             0x3e,
             0x4b,
             0x50);

com_interface! {
    interface ISRGramCFG : IUnknown {
        iid: IID_ISRGramCFG,
        vtable: ISRGramCFGVtable,
        fn link_query(list_name: BStr, result: *mut i32) -> HRESULT;
        fn list_append(list_name: BStr, word: SDATA) -> HRESULT;
        fn list_get(list_name: BStr, word: *mut SDATA) -> HRESULT;
        fn list_remove(list_name: BStr, word: SDATA) -> HRESULT;
        fn list_set(list_name: BStr, word: SDATA) -> HRESULT;
        fn list_query(list_name: BStr, result: *mut i32) -> HRESULT;
    }
}

define_guid!(IID_IDgnSRGramSelect = 0xdd10901a,
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
    interface IDgnSRGramSelect : IUnknown {
        iid: IID_IDgnSRGramSelect,
        vtable: IDgnSRGramSelectVtable,
        fn words_set(words: SDATA) -> HRESULT;
        fn words_change(start: u32, stop: u32, words: SDATA) -> HRESULT;
        fn words_delete(start: u32, stop: u32) -> HRESULT;
        fn words_insert(start: u32, words: SDATA) -> HRESULT;
        fn words_get(words: *mut SDATA) -> HRESULT;
    }
}

define_guid!(IID_IDgnSRResSelect = 0xdd10801b,
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
    interface IDgnSRResSelect : IUnknown {
        iid: IID_IDgnSRResSelect,
        vtable: IDgnSRResSelectVtable,
        fn get_info(guid: GUID, choice: u32, start: *mut u32, stop: *mut u32, word_num: *mut u32) -> HRESULT;
    }
}

define_guid!(IID_ISRGramDictation = 0x090CD9A3,
             0xDA1A,
             0x11CD,
             0xB3,
             0xCA,
             0x0,
             0xAA,
             0x0,
             0x47,
             0xBA,
             0x4F);

com_interface! {
    interface ISRGramDictation : IUnknown {
        iid: IID_ISRGramDictation,
        vtable: ISRGramDictationVtable,
        fn context(before: BStr, after: BStr) -> HRESULT;
        fn hint(hint: BStr) -> HRESULT;
        fn words(words: BStr) -> HRESULT;
    }
}
