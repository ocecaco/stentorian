use crate::dragon::{RECEIVE_SDATA, SDATA, SRWORD};
use crate::errors::*;
use crate::interfaces::{IDgnSRGramSelect, ISRGramCFG, ISRGramCommon, ISRGramDictation};
use byteorder::{LittleEndian, WriteBytesExt};
use components::bstr::{BStr, BString};
use components::comptr::ComPtr;
use components::Cast;
use std::mem;
use std::ptr;
use std::slice;

pub struct CommandGrammarControl {
    grammar_control: ComPtr<ISRGramCommon>,
    grammar_lists: ComPtr<ISRGramCFG>,
}

pub fn create_command(grammar_control: ComPtr<ISRGramCommon>) -> Result<CommandGrammarControl> {
    let grammar_lists = grammar_control.cast()?;

    Ok(CommandGrammarControl {
        grammar_control: grammar_control,
        grammar_lists: grammar_lists,
    })
}

impl CommandGrammarControl {
    pub fn rule_activate(&self, name: &str) -> Result<()> {
        let rc = unsafe {
            self.grammar_control
                .activate(ptr::null_mut(), 0, BString::from(name).as_ref())
        };

        rc.result()?;
        Ok(())
    }

    pub fn rule_deactivate(&self, name: &str) -> Result<()> {
        let rc = unsafe {
            self.grammar_control
                .deactivate(BString::from(name).as_ref())
        };

        rc.result()?;
        Ok(())
    }

    pub fn list_append(&self, name: &str, word: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = word.into();
        let data = word_into_data(&srword);

        let rc = unsafe { self.grammar_lists.list_append(name.as_ref(), data) };

        rc.result()?;
        Ok(())
    }

    pub fn list_remove(&self, name: &str, word: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = word.into();
        let data = word_into_data(&srword);

        let rc = unsafe { self.grammar_lists.list_remove(name.as_ref(), data) };

        rc.result()?;
        Ok(())
    }

    pub fn list_clear(&self, name: &str) -> Result<()> {
        let name = BString::from(name);
        let srword: SRWORD = "".into();
        let data = word_into_data(&srword);

        let rc = unsafe { self.grammar_lists.list_set(name.as_ref(), data) };

        rc.result()?;
        Ok(())
    }
}

struct GrammarActivation(ComPtr<ISRGramCommon>, Option<BString>);

impl GrammarActivation {
    fn new(ptr: ComPtr<ISRGramCommon>, name: Option<&str>) -> Self {
        GrammarActivation(ptr, name.map(|n| BString::from(n)))
    }

    fn name(&self) -> BStr {
        if let Some(ref n) = self.1 {
            return n.as_ref();
        }

        BStr::null()
    }

    fn activate(&self) -> Result<()> {
        let rc = unsafe { self.0.activate(ptr::null_mut(), 0, self.name()) };

        rc.result()?;
        Ok(())
    }

    fn deactivate(&self) -> Result<()> {
        let rc = unsafe { self.0.deactivate(self.name()) };

        rc.result()?;
        Ok(())
    }
}

pub struct SelectGrammarControl {
    grammar_activation: GrammarActivation,
    grammar_select: ComPtr<IDgnSRGramSelect>,
}

pub fn create_select(grammar_control: ComPtr<ISRGramCommon>) -> Result<SelectGrammarControl> {
    let grammar_select = grammar_control.cast()?;

    Ok(SelectGrammarControl {
        grammar_activation: GrammarActivation::new(grammar_control, None),
        grammar_select: grammar_select,
    })
}

impl SelectGrammarControl {
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

pub fn create_dictation(grammar_control: ComPtr<ISRGramCommon>) -> Result<DictationGrammarControl> {
    let grammar_dictation = grammar_control.cast()?;

    Ok(DictationGrammarControl {
        grammar_activation: GrammarActivation::new(grammar_control, None),
        grammar_dictation: grammar_dictation,
    })
}

impl DictationGrammarControl {
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

pub fn create_catchall(grammar_control: ComPtr<ISRGramCommon>) -> Result<CatchallGrammarControl> {
    Ok(CatchallGrammarControl {
        grammar_activation: GrammarActivation::new(grammar_control, Some("dummy")),
    })
}

impl CatchallGrammarControl {
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
