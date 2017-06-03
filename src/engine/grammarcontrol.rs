use components::comptr::ComPtr;
use components::bstr::BString;
use errors::*;
use super::grammarsink::interfaces::{ISRGramCommon, ISRGramCFG};
use std::ptr;
use std::mem;
use dragon::*;

pub fn create(grammar_control: ComPtr<ISRGramCommon>,
              grammar_lists: ComPtr<ISRGramCFG>)
              -> GrammarControl {
    GrammarControl {
        grammar_control: grammar_control,
        grammar_lists: grammar_lists,
    }
}

pub struct GrammarControl {
    grammar_control: ComPtr<ISRGramCommon>,
    grammar_lists: ComPtr<ISRGramCFG>,
}

impl GrammarControl {
    pub fn rule_activate(&self, name: &str) -> Result<()> {
        let rc = unsafe {
            self.grammar_control
                .activate(ptr::null(), 0, BString::from(name).as_ref())
        };

        try!(rc.result());
        Ok(())
    }

    pub fn rule_deactivate(&self, name: &str) -> Result<()> {
        let rc = unsafe {
            self.grammar_control
                .deactivate(BString::from(name).as_ref())
        };

        try!(rc.result());
        Ok(())
    }

    pub fn list_append(&self, name: &str, word: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = word.into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { self.grammar_lists.list_append(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }

    pub fn list_remove(&self, name: &str, word: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = word.into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { self.grammar_lists.list_remove(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }

    pub fn list_clear(&self, name: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = "".into();

        let data = SDATA {
            data: &srword as *const SRWORD as *const u8,
            size: mem::size_of::<SRWORD>() as u32,
        };

        let rc = unsafe { self.grammar_lists.list_set(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }
}
