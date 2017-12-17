use components::comptr::ComPtr;
use components::bstr::BString;
use components::*;
use errors::*;
use interfaces::*;
use std::ptr;
use std::slice;
use std::mem;
use byteorder::{LittleEndian, WriteBytesExt};
use dragon::*;

pub fn create(grammar_control: ComPtr<ISRGramCommon>)
              -> Result<GrammarControl> {
    let grammar_lists = query_interface::<ISRGramCFG>(&grammar_control)
        .ok()
        .map(|v| GrammarLists { grammar_lists: v });

    let grammar_select = query_interface::<IDgnSRGramSelect>(&grammar_control)
        .ok()
        .map(|v| GrammarSelect { grammar_select: v });

    let grammar_dragon = query_interface::<IDgnSRGramCommon>(&grammar_control)?;

    let mut guid: GUID = unsafe { mem::uninitialized() };
    let rc = unsafe {
        grammar_dragon.identify(&mut guid)
    };
    rc.result()?;

    Ok(GrammarControl {
        guid: guid,
        grammar_control: grammar_control,
        grammar_lists: grammar_lists,
        grammar_select: grammar_select,
    })
}

pub struct GrammarControl {
    guid: GUID,
    grammar_control: ComPtr<ISRGramCommon>,
    grammar_lists: Option<GrammarLists>,
    grammar_select: Option<GrammarSelect>,
}

impl GrammarControl {
    pub fn guid(&self) -> GUID {
        self.guid
    }

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
    pub fn text_set(&self, text: &str) -> Result<()> {
        let encoded = encode_text(text);

        let rc = unsafe { self.grammar_select.words_set(encoded.as_slice().into()) };
        rc.result()?;

        Ok(())
    }

    pub fn text_change(&self, start: u32, stop: u32, text: &str) -> Result<()> {
        let encoded = encode_text(text);

        let rc = unsafe { self.grammar_select.words_change(start, stop, encoded.as_slice().into()) };
        rc.result()?;

        Ok(())
    }

    pub fn text_delete(&self, start: u32, stop: u32) -> Result<()> {
        let rc = unsafe { self.grammar_select.words_delete(start, stop) };
        rc.result()?;

        Ok(())
    }

    pub fn text_insert(&self, start: u32, text: &str) -> Result<()> {
        let encoded = encode_text(text);

        let rc = unsafe { self.grammar_select.words_insert(start, encoded.as_slice().into()) };
        rc.result()?;

        Ok(())
    }

    pub fn text_get(&self) -> Result<String> {
        let mut data = RECEIVE_SDATA::new();

        let rc = unsafe { self.grammar_select.words_get(&mut data) };
        rc.result()?;

        let slice = data.as_slice().unwrap();
        let slice_16 = unsafe { slice::from_raw_parts(slice.as_ptr() as *const u16, slice.len() / 2) };
        let (_, without_terminator) = slice_16.split_last().unwrap();

        let text = String::from_utf16_lossy(without_terminator);

        Ok(text)
    }
}


fn encode_text(text: &str) -> Vec<u8> {
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
    encoded
}

fn word_into_data<'a>(word: &'a SRWORD) -> SDATA<'a> {
    let ptr = word as *const SRWORD as *const u8;
    let count = mem::size_of::<SRWORD>();
    let slice: &'a [u8] = unsafe { slice::from_raw_parts(ptr, count) };

    slice.into()
}
