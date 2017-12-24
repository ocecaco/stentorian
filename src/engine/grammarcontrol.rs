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

pub struct CommandGrammarControl {
    grammar_control: ComPtr<ISRGramCommon>,
    grammar_lists: ComPtr<ISRGramCFG>,
}

impl CommandGrammarControl {
    pub fn create(grammar_control: ComPtr<ISRGramCommon>) -> Result<Self> {
        let grammar_lists = query_interface::<ISRGramCFG>(&grammar_control)?;

        Ok(CommandGrammarControl {
            grammar_control: grammar_control,
            grammar_lists: grammar_lists,
        })
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

struct GrammarActivation(ComPtr<ISRGramCommon>);

impl GrammarActivation {
    fn activate(&self) -> Result<()> {
        let rc = unsafe { self.0.activate(ptr::null(), 0, BString::from("").as_ref()) };

        try!(rc.result());
        Ok(())
    }

    fn deactivate(&self) -> Result<()> {
        let rc = unsafe { self.0.deactivate(BString::from("").as_ref()) };

        try!(rc.result());
        Ok(())
    }
}

pub struct SelectGrammarControl {
    grammar_activation: GrammarActivation,
    grammar_select: ComPtr<IDgnSRGramSelect>,
}

impl SelectGrammarControl {
    pub fn create(grammar_control: ComPtr<ISRGramCommon>) -> Result<Self> {
        let grammar_select = query_interface::<IDgnSRGramSelect>(&grammar_control)?;

        Ok(SelectGrammarControl {
            grammar_activation: GrammarActivation(grammar_control),
            grammar_select: grammar_select,
        })
    }

    pub fn activate(&self) -> Result<()> {
        self.grammar_activation.activate()
    }

    pub fn deactivate(&self) -> Result<()> {
        self.grammar_activation.deactivate()
    }

    pub fn text_set(&self, text: &str) -> Result<()> {
        let encoded = encode_text(text);

        let rc = unsafe { self.grammar_select.words_set(encoded.as_slice().into()) };
        rc.result()?;

        Ok(())
    }

    pub fn text_change(&self, start: u32, stop: u32, text: &str) -> Result<()> {
        let encoded = encode_text(text);

        let rc = unsafe {
            self.grammar_select
                .words_change(start, stop, encoded.as_slice().into())
        };
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

        let rc = unsafe {
            self.grammar_select
                .words_insert(start, encoded.as_slice().into())
        };
        rc.result()?;

        Ok(())
    }

    pub fn text_get(&self) -> Result<String> {
        let mut data = RECEIVE_SDATA::new();

        let rc = unsafe { self.grammar_select.words_get(&mut data) };
        rc.result()?;

        let slice = data.as_slice().unwrap();
        let slice_16 =
            unsafe { slice::from_raw_parts(slice.as_ptr() as *const u16, slice.len() / 2) };
        let (_, without_terminator) = slice_16.split_last().unwrap();

        let text = String::from_utf16_lossy(without_terminator);

        Ok(text)
    }
}

pub struct DictationGrammarControl {
    grammar_activation: GrammarActivation,
    grammar_dictation: ComPtr<ISRGramDictation>,
}

impl DictationGrammarControl {
    pub fn create(grammar_control: ComPtr<ISRGramCommon>) -> Result<Self> {
        let grammar_dictation = query_interface::<ISRGramDictation>(&grammar_control)?;

        Ok(DictationGrammarControl {
            grammar_activation: GrammarActivation(grammar_control),
            grammar_dictation: grammar_dictation,
        })
    }

    pub fn activate(&self) -> Result<()> {
        self.grammar_activation.activate()
    }

    pub fn deactivate(&self) -> Result<()> {
        self.grammar_activation.deactivate()
    }

    pub fn context_set(&self, context: &str) -> Result<()> {
        let rc = unsafe {
            self.grammar_dictation
                .context(BString::from(context).as_ref(), BString::from("").as_ref())
        };
        rc.result()?;

        Ok(())
    }
}

pub struct CatchallGrammarControl {
    grammar_activation: GrammarActivation,
}

impl CatchallGrammarControl {
    pub fn create(grammar_control: ComPtr<ISRGramCommon>) -> Result<Self> {
        Ok(CatchallGrammarControl {
            grammar_activation: GrammarActivation(grammar_control),
        })
    }

    pub fn activate(&self) -> Result<()> {
        self.grammar_activation.activate()
    }

    pub fn deactivate(&self) -> Result<()> {
        self.grammar_activation.deactivate()
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
