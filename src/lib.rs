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

use errors::*;

pub fn initialize() -> Result<()> {
    components::com_initialize()?;

    Ok(())
}
