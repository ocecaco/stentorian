#![cfg(windows)]
#![cfg(target_arch = "x86")]
#![cfg(target_env = "msvc")]
#![allow(dead_code)]
extern crate byteorder;

#[macro_use]
extern crate bitflags;

extern crate failure;
#[macro_use]
extern crate failure_derive;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
extern crate components;

pub mod errors {
    use ::components::errors::ComError;
    use ::grammarcompiler::errors::GrammarError;

    pub type Result<T> = ::std::result::Result<T, Error>;

    #[derive(Fail, Debug)]
    pub enum Error {
        #[fail(display = "{}", _0)]
        Com(#[cause] ComError),
        #[fail(display = "{}", _0)]
        Grammar(#[cause] GrammarError),
        #[fail(display = "attempt to perform operation on unloaded grammar")]
        GrammarGone,
    }

    impl From<ComError> for Error {
        fn from(e: ComError) -> Error {
            Error::Com(e)
        }
    }

    impl From<GrammarError> for Error {
        fn from(e: GrammarError) -> Error {
            Error::Grammar(e)
        }
    }
}

pub mod engine;
pub mod grammar;
pub mod resultparser;

mod interfaces;
mod dragon;
mod grammarcompiler;

use errors::*;

pub fn initialize() -> Result<()> {
    components::com_initialize()?;

    Ok(())
}
