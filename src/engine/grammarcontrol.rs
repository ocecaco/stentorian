use components::comptr::ComPtr;
use components::bstr::BString;
use components::*;
use errors::*;
use super::grammarsink::interfaces::{ISRGramCommon, ISRGramCFG};
use super::interfaces::IDgnSRGramSelect;
use std::ptr;
use std::mem;
use byteorder::{LittleEndian, WriteBytesExt};
use dragon::*;

pub fn create(grammar_control: ComPtr<ISRGramCommon>)
              -> GrammarControl {
    let grammar_lists = query_interface::<ISRGramCFG>(&grammar_control)
        .ok()
        .map(|v| GrammarLists { grammar_lists: v });

    let grammar_select = query_interface::<IDgnSRGramSelect>(&grammar_control)
        .ok()
        .map(|v| GrammarSelect { grammar_select: v });

    GrammarControl {
        grammar_control: grammar_control,
        grammar_lists: grammar_lists,
        grammar_select: grammar_select,
    }
}

pub struct GrammarControl {
    grammar_control: ComPtr<ISRGramCommon>,
    grammar_lists: Option<GrammarLists>,
    grammar_select: Option<GrammarSelect>,
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

    pub fn lists(&self) -> Option<&GrammarLists> {
        self.grammar_lists.as_ref()
    }

    pub fn select_context(&self) -> Option<&GrammarSelect> {
        self.grammar_select.as_ref()
    }
}

pub struct GrammarLists {
    grammar_lists: ComPtr<ISRGramCFG>,
}

impl GrammarLists {
    pub fn list_append(&self, name: &str, word: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = word.into();
        let data = word_into_data(&srword);

        let rc = unsafe { self.grammar_lists.list_append(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }

    pub fn list_remove(&self, name: &str, word: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = word.into();
        let data = word_into_data(&srword);

        let rc = unsafe { self.grammar_lists.list_remove(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }

    pub fn list_clear(&self, name: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = "".into();
        let data = word_into_data(&srword);

        let rc = unsafe { self.grammar_lists.list_set(name.as_ref(), data) };

        try!(rc.result());
        Ok(())
    }
}

pub struct GrammarSelect {
    grammar_select: ComPtr<IDgnSRGramSelect>,
}

impl GrammarSelect {
    pub fn words_set(&self, text: &str) -> Result<()> {
        fn add_padding(v: &mut Vec<u8>, multiple: usize) {
            let extra_padding = multiple - (v.len() % multiple);
            for _ in 0..extra_padding {
                v.push(0u8);
            }
        }

        fn encode(s: &str) -> Vec<u8> {
            let mut result = Vec::new();
            for c in s.encode_utf16() {
                result.write_u16::<LittleEndian>(c).unwrap();
            }
            result
        }

        let mut encoded = encode(text);

        // make sure word is terminated by at least *two* null bytes
        // after padding
        encoded.push(0u8);
        add_padding(&mut encoded, 4);

        let data = SDATA {
            data: encoded.as_ptr(),
            size: encoded.len() as u32,
        };

        let rc = unsafe { self.grammar_select.words_set(data) };
        rc.result()?;

        Ok(())
    }
}

fn word_into_data(word: &SRWORD) -> SDATA {
    SDATA {
        data: word as *const SRWORD as *const u8,
        size: mem::size_of::<SRWORD>() as u32,
    }
}
