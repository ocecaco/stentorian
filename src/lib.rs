#![allow(dead_code)]
#![allow(unused_variables)]
extern crate libc;

#[macro_use(bitflags)]
extern crate bitflags;

extern crate byteorder;

extern crate encoding;

#[macro_use]
mod macros;

mod comutil;
mod bstr;
mod comptr;
mod iunknown;
mod types;
mod refcount;
mod grammarsink;
mod enginesink;
mod grammarcompiler;
mod grammar;
mod resultparser;

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
        szDialect: [u16; LANG_LEN],
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
        dwEngineFeatures: u32,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    #[allow(non_camel_case_types)]
    pub enum SRGRMFMT {
        SRGRMFMT_CFG = 0x0000,
        SRGRMFMT_LIMITEDDOMAIN = 0x0001,
        SRGRMFMT_DICTATION = 0x0002,
        SRGRMFMT_CFGNATIVE = 0x8000,
        SRGRMFMT_LIMITEDDOMAINNATIVE = 0x8001,
        SRGRMFMT_DICTATIONNATIVE = 0x8002,
        SRGRMFMT_DRAGONNATIVE1 = 0x8101,
        SRGRMFMT_DRAGONNATIVE2 = 0x8102,
        SRGRMFMT_DRAGONNATIVE3 = 0x8103,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct SDATA {
        pub data: *const u8,
        pub size: u32,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    #[allow(non_camel_case_types)]
    pub enum VOICEPARTOFSPEECH {
        VPS_UNKNOWN = 0,
        VPS_NOUN = 1,
        VPS_VERB = 2,
        VPS_ADVERB = 3,
        VPS_ADJECTIVE = 4,
        VPS_PROPERNOUN = 5,
        VPS_PRONOUN = 6,
        VPS_CONJUNCTION = 7,
        VPS_CARDINAL = 8,
        VPS_ORDINAL = 9,
        VPS_DETERMINER = 10,
        VPS_QUANTIFIER = 11,
        VPS_PUNCTUATION = 12,
        VPS_CONTRACTION = 13,
        VPS_INTERJECTION = 14,
        VPS_ABBREVIATION = 15,
        VPS_PREPOSITION = 16,
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct SRRESWORDNODE {
        dwNextWordNode: u32,
        dwUpAlternateWordNode: u32,
        dwDownAlternateWordNode: u32,
        dwPreviousWordNode: u32,
        dwPhonemeNode: u32,
        qwStartTime: u64,
        qwEndTime: u64,
        dwWordScore: u32,
        wVolume: u16,
        wPitch: u16,
        pos: VOICEPARTOFSPEECH,
        pub dwCFGParse: u32,
        dwCue: u32,
    }

    #[repr(C)]
    pub struct SRWORD {
        pub size: u32,
        pub word_number: u32,
        pub buffer: [u16; 128],
    }

    define_guid!(pub CLSID_DgnDictate = 0xdd100001,
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

    define_guid!(pub CLSID_DgnSite = 0xdd100006,
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
}


mod iserviceprovider {
    use super::types::*;
    use super::iunknown::*;

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
}

mod isrcentral {
    use super::types::*;
    use super::iunknown::*;
    use super::dragon::*;
    use super::bstr::*;
    use libc::c_void;

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
            fn identify(g: *const GUID) -> HRESULT;
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
}


mod api {
    use types::*;
    use std::ptr;
    use super::dragon::*;
    use super::iunknown::*;
    use super::iserviceprovider::*;
    use super::isrcentral::*;
    use super::comptr::*;
    use super::grammarsink::*;
    use super::enginesink;
    use super::bstr::*;

    use comutil::*;

    use grammar::*;
    use grammarcompiler::*;

    use std::io;

    use resultparser;

    fn get_engine(provider: &IServiceProvider) -> ComPtr<ISRCentral> {
        unsafe {
            let mut central: RawComPtr = ptr::null();
            let result =
                provider.query_service(&CLSID_DgnDictate, &ISRCentral::iid(), &mut central);
            assert_eq!(result.0, 0);
            raw_to_comptr::<ISRCentral>(central, true)
        }
    }

    fn test_grammar_load(engine: &ISRCentral, grammar: &[u8]) -> ComPtr<ISRGramCommon> {
        let mut control: RawComPtr = ptr::null();
        let result = unsafe {
            engine.grammar_load(SRGRMFMT::SRGRMFMT_CFG,
                                SDATA {
                                    data: grammar.as_ptr(),
                                    size: grammar.len() as u32,
                                },
                                make_grammar_sink(),
                                ISRGramNotifySink::iid(),
                                &mut control)
        };
        assert_eq!(result.0, 0);

        let grammar_control = unsafe { raw_to_comptr::<IUnknown>(control, true) };
        let grammar_control = query_interface::<ISRGramCommon>(&grammar_control).unwrap();
        unsafe {
            let result =
                grammar_control.activate(ptr::null(), 0, BString::from("Mapping").as_ref());
            assert_eq!(result.0 as u32, 0);
        }

        grammar_control
    }

    fn create_engine_sink(engine: ComPtr<IDgnSREngineControl>) -> ComPtr<IDgnSREngineNotifySink> {
        let sink = enginesink::make_engine_sink(engine);
        let sink = unsafe { raw_to_comptr::<ISRNotifySink>(sink, true) };
        query_interface::<IDgnSREngineNotifySink>(&sink).unwrap()
    }

    fn do_grammar_test() {
        let provider =
            create_instance::<IServiceProvider>(&CLSID_DgnSite, None, CLSCTX_LOCAL_SERVER).unwrap();

        let engine = get_engine(&provider);
        let engine_control = query_interface::<IDgnSREngineControl>(&engine).unwrap();

        let mut key = 0u32;
        let sink = create_engine_sink(engine_control);
        let result = unsafe {
            engine.register(&sink as &IDgnSREngineNotifySink as *const _ as RawComPtr,
                            IDgnSREngineNotifySink::iid(),
                            &mut key)
        };
        assert_eq!(result.0, 0);

        // let grammar = make_test_grammar();

        // let matcher = resultparser::compiler::compile_grammar_matcher(&grammar);
        // let result = resultparser::vm::perform_match(&matcher, &["hello", "one", "three", "world"]);
        // println!("{:?}", result);
        // for (i, x) in matcher.iter().enumerate() {
        //     if let resultparser::instructions::Instruction::NoOp = *x {
        //         println!("{}:", i);
        //     } else {
        //         println!("{}: {:?}", i, x);
        //     }
        // }

        // let compiled = compile_grammar(&grammar);
        // let control = test_grammar_load(&engine, &compiled);

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }

    pub fn test() {
        unsafe {
            let result: HRESULT = CoInitializeEx(ptr::null(), COINIT_MULTITHREADED);
            assert_eq!(result.0, 0);
        }

        do_grammar_test();

        unsafe {
            CoUninitialize();
        }
    }
}

pub fn main() {
    api::test();
}
