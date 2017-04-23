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

pub mod errors {
    error_chain! {
        links {
            Com(::components::errors::Error, ::components::errors::ErrorKind);
            Grammar(::grammarcompiler::errors::Error, ::grammarcompiler::errors::ErrorKind);
        }
    }
}

pub mod engine;
pub mod grammar;

mod interfaces;
mod dragon;
mod resultparser;
mod grammarcompiler;

use grammar::Grammar;
use engine::*;
use components::*;
use std::sync::mpsc;
use errors::*;
use resultparser::Matcher;

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
                    {
                        "type": "word",
                        "text": "beautiful"
                    },
                    {
                        "type": "capture",
                        "key": "testing123",
                        "child": {
                            "type": "dictation"
                        }
                    }
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

fn test() -> Result<()> {
    let _com = unsafe { com_initialize()? };

    let engine = Engine::connect()?;
    let (tx, rx) = mpsc::channel();
    let _registration = engine
        .register(SEND_PAUSED | SEND_ATTRIBUTE, tx.clone())?;

    let grammar = make_test_grammar();
    let grammar_control =
        engine
            .grammar_load(SEND_PHRASE_FINISH | SEND_FOREIGN_FINISH, &grammar, tx)?;
    let matcher = Matcher::new(&grammar);

    grammar_control.rule_activate("Mapping")?;

    for _ in 0..20 {
        match rx.recv().unwrap() {
            Event::Engine(EngineEvent::Paused(cookie)) => {
                println!("paused");
                engine.resume(cookie)?;
            }
            Event::Grammar(GrammarEvent::PhraseFinish(words)) => {
                println!("{:?}", words);
                println!("{:?}", matcher.perform_match(&words));
            }
            Event::Engine(EngineEvent::AttributeChanged(a)) => {
                println!("{:?}", a);
                println!("{:?}", engine.microphone_get_state()?);
            }
            _ => println!("something else"),
        }
    }

    Ok(())
}

quick_main!(test);

pub fn external_main() {
    main();
}
