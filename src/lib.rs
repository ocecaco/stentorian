#![cfg(windows)]
#![cfg(target_arch = "x86")]
#![cfg(target_env = "msvc")]
pub mod errors {
    use crate::grammarcompiler::errors::GrammarError;
    use components::errors::ComError;
    use failure::Fail;

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

mod dragon;
mod grammarcompiler;
mod interfaces;

use errors::*;

pub fn initialize() -> Result<()> {
    components::com_initialize()?;

    Ok(())
}
