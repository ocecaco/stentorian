#![allow(dead_code)]
extern crate byteorder;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate components;

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {
        links {
            Com(::components::errors::Error, ::components::errors::ErrorKind);
            Grammar(::engine::grammar_errors::Error, ::engine::grammar_errors::ErrorKind);
        }
    }
}

mod interfaces;
mod dragon;
mod engine;
mod grammar;
mod resultparser;

use grammar::Grammar;
use engine::*;
use components::*;
use std::sync::mpsc;
use errors::*;

fn make_test_grammar() -> Grammar {
    let data = r#"
{
    "rules": [
        {
            "name": "Mapping",
            "exported": true,
            "definition": {
                    "type": "list",
                    "name": "testlist"
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

fn test() -> Result<()> {
    let _com = unsafe { com_initialize()? };

    let engine = Engine::connect()?;
    let (tx, rx) = mpsc::channel();
    let _registration = engine.register(SEND_PAUSED | SEND_ATTRIBUTE, tx.clone())?;

    let grammar = make_test_grammar();
    let grammar_control = engine.grammar_load(SEND_PHRASE_FINISH | SEND_FOREIGN_FINISH, &grammar, tx)?;

    grammar_control.rule_activate("Mapping")?;

    grammar_control.list_append("testlist", "bazerong")?;
    grammar_control.list_append("testlist", "ookabooka")?;
    grammar_control.list_clear("testlist")?;
    grammar_control.list_append("testlist", "Visual Studio")?;

    for _ in 0..100 {
        match rx.recv().unwrap() {
            Event::Engine(EngineEvent::Paused(cookie)) => {
                println!("paused");
                engine.resume(cookie)?;
            }
            Event::Grammar(GrammarEvent::PhraseFinish(words)) => {
                println!("{:?}", words);
            }
            Event::Engine(EngineEvent::AttributeChanged(a)) => {
                println!("{:?}", a);
                println!("{:?}", engine.microphone_get_state()?);
            },
            _ => println!("something else"),
        }
    }

    Ok(())
}

quick_main!(test);

pub fn external_main() {
    main();
}
