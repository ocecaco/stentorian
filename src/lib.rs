#![allow(dead_code)]
extern crate byteorder;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate components;

#[macro_use]
extern crate error_chain;

extern crate futures;

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
use std::fs::File;
use std::io::Read;
use errors::*;
use futures::Stream;

fn make_test_grammar() -> Grammar {
    let mut file = File::open("test.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    serde_json::from_str(&contents).unwrap()
}

fn test() -> Result<()> {
    env_logger::init().unwrap();

    let _com = unsafe { com_initialize()? };

    let engine = Engine::connect()?;

    let registration = engine.register()?;

    let test = registration.take(10).map(|event| {
        println!("{:?}", event);
        if let EngineEvent::Paused(cookie) = event {
            engine.resume(cookie).unwrap();
        }
    });

    for _ in test.wait() {
    }

    Ok(())
}

quick_main!(test);

pub fn external_main() {
    main();
}
