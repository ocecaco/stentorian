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
mod engine;
mod grammar;
mod resultparser;

use grammar::Grammar;
use engine::*;
use components::*;
use std::ptr;

fn make_test_grammar() -> Grammar {
    let data = r#"
{
    "rules": [
        {
            "name": "Mapping",
            "definition": {
                "exported": true,
                "element": {
                    "type": "sequence",
                    "children": [
                        {"type": "word", "text": "hello"},
                        {"type": "word", "text": "testing"},
                        {"type": "word", "text": "beautiful"},
                        {"type": "word", "text": "soup"}
                    ]
                }
            }
        }
    ]
}
"#;

    serde_json::from_str(data).unwrap()
}


fn test() {
    let engine = Engine::connect();
    let receiver = engine.register(SEND_BEGIN_UTTERANCE);

    let grammar = make_test_grammar();
    let grammar_control = engine.grammar_load(SEND_PHRASE_FINISH,
                                              &grammar);
    grammar_control.activate_rule("Mapping");

    for _ in 0..10 {
        match grammar_control.recv().unwrap() {
            GrammarEvent::PhraseFinish(words) => {
                println!("{:?}", words);
            },
            _ => println!("something else"),
        }
    }
}

pub fn main() {
    unsafe {
        let result = CoInitializeEx(ptr::null(), COINIT_MULTITHREADED);
        assert_eq!(result.0, 0);
    }

    test();

    unsafe {
        CoUninitialize();
    }
}
