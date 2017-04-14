#![allow(dead_code)]
#![allow(unused_variables)]

extern crate byteorder;
extern crate encoding;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate components;

mod interfaces;
mod dragon;
mod grammarsink;
mod engine;
mod grammarcompiler;
mod grammar;
mod resultparser;

mod api {
//     use components::*;
//     use std::ptr;
//     use super::dragon::*;
//     use super::iserviceprovider::*;
//     use super::isrcentral::*;
//     use components::comptr::*;
//     use super::grammarsink::*;
//     use super::enginesink;
//     use components::bstr::*;

//     use grammar::*;
//     use grammarcompiler::*;

//     use std::io;

//     use serde_json;

//     use resultparser;

//     fn test_grammar_load(engine: &ISRCentral, grammar: &[u8]) -> ComPtr<ISRGramCommon> {
//         let mut control: RawComPtr = ptr::null();
//         let result = unsafe {
//             engine.grammar_load(SRGRMFMT::SRGRMFMT_CFG,
//                                 SDATA {
//                                     data: grammar.as_ptr(),
//                                     size: grammar.len() as u32,
//                                 },
//                                 make_grammar_sink(),
//                                 ISRGramNotifySink::iid(),
//                                 &mut control)
//         };
//         assert_eq!(result.0, 0);

//         let grammar_control = unsafe { raw_to_comptr::<IUnknown>(control, true) };
//         let grammar_control = query_interface::<ISRGramCommon>(&grammar_control).unwrap();
//         unsafe {
//             let result =
//                 grammar_control.activate(ptr::null(), 0, BString::from("Mapping").as_ref());
//             assert_eq!(result.0 as u32, 0);
//         }

//         grammar_control
//     }

//     fn create_engine_sink(engine: ComPtr<IDgnSREngineControl>) -> ComPtr<IDgnSREngineNotifySink> {
//         let sink = enginesink::make_engine_sink(engine);
//         let sink = unsafe { raw_to_comptr::<ISRNotifySink>(sink, true) };
//         query_interface::<IDgnSREngineNotifySink>(&sink).unwrap()
//     }

//     fn make_test_grammar() -> Grammar {
//         let data = r#"
// {
//     "rules": [
//         {
//             "name": "Mapping",
//             "definition": {
//                 "exported": true,
//                 "element": {
//                     "type": "sequence",
//                     "children": [
//                         {"type": "word", "text": "hello"},
//                         {"type": "word", "text": "testing"},
//                         {"type": "word", "text": "beautiful"},
//                         {"type": "word", "text": "soup"}
//                     ]
//                 }
//             }
//         }
//     ]
// }
// "#;

//         serde_json::from_str(data).unwrap()
//     }

//     fn do_grammar_test() {
//         let mut key = 0u32;
//         let sink = create_engine_sink(engine_control);
//         let result = unsafe {
//             engine.register(&sink as &IDgnSREngineNotifySink as *const _ as RawComPtr,
//                             IDgnSREngineNotifySink::iid(),
//                             &mut key)
//         };
//         assert_eq!(result.0, 0);

//         let grammar = make_test_grammar();

//         let matcher = resultparser::compiler::compile_grammar_matcher(&grammar);
//         let result = resultparser::vm::perform_match(&matcher, &["hello", "one", "three", "world"]);
//         println!("{:?}", result);
//         for (i, x) in matcher.iter().enumerate() {
//             if let resultparser::instructions::Instruction::NoOp = *x {
//                 println!("{}:", i);
//             } else {
//                 println!("{}: {:?}", i, x);
//             }
//         }

//         let compiled = compile_grammar(&grammar);
//         let control = test_grammar_load(&engine, &compiled);

//         let mut input = String::new();
//         io::stdin().read_line(&mut input).unwrap();
//     }

    // pub fn test() {
    //     unsafe {
    //         let result: HRESULT = CoInitializeEx(ptr::null(), COINIT_MULTITHREADED);
    //         assert_eq!(result.0, 0);
    //     }

    //     unsafe {
    //         CoUninitialize();
    //     }
    // }
}

pub fn main() {
}
