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
use std::sync::mpsc;

fn make_test_grammar() -> Grammar {
    let data = r#"
{
    "rules": [
        {
            "name": "Mapping",
            "exported": true,
            "definition": {
                    "type": "sequence",
                    "children": [
                        {"type": "word", "text": "hello"},
                        {"type": "word", "text": "testing"},
                        {"type": "alternative", "children": [
                            {"type": "dictation_word"},
                            {"type": "word", "text": "world"}
                        ]},
                        {"type": "word", "text": "soup"}
                    ]
                }
        }
    ]
}
"#;

    serde_json::from_str(data).unwrap()
}

#[derive(Debug)]
enum Event {
    Engine(EngineEvent),
    Grammar(GrammarEvent),
}

impl From<EngineEvent> for Event {
    fn from(event: EngineEvent) -> Self {
        Event::Engine(event)
    }
}

impl From<GrammarEvent> for Event {
    fn from(event: GrammarEvent) -> Self {
        Event::Grammar(event)
    }
}

fn test() {
    let engine = Engine::connect();
    let (tx, rx) = mpsc::channel();
    let registration = engine.register(SEND_PAUSED | SEND_ATTRIBUTE, tx.clone());

    let grammar = make_test_grammar();
    let grammar_control = engine.grammar_load(SEND_PHRASE_FINISH, &grammar, tx);

    grammar_control.activate_rule("Mapping");

    for _ in 0..10 {
        match rx.recv().unwrap() {
            Event::Engine(EngineEvent::Paused(cookie)) => {
                println!("paused");
                engine.resume(cookie);
            }
            Event::Grammar(GrammarEvent::PhraseFinish(words)) => {
                println!("{:?}", words);
            }
            Event::Engine(EngineEvent::AttributeChanged(a)) => println!("{:?}", a as u32),
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
