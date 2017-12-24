use std::mem;
use dragon::*;
use interfaces::*;
use components::*;
use errors::*;
use super::events::GrammarEvent;

const VALUE_OUT_OF_RANGE: u32 = 0x8000_FFFF;

pub type CommandGrammarEvent = GrammarEvent<Vec<Words>>;
pub type Words = Vec<WordInfo>;

pub struct WordInfo {
    pub text: String,
    pub rule: u32,
    pub start_time: u64,
    pub end_time: u64,
}

pub type Selection = (u32, u32);
pub type SelectGrammarEvent = GrammarEvent<Vec<Selection>>;

fn string_from_slice(s: &[u16]) -> String {
    String::from_utf16_lossy(&s.iter()
        .cloned()
        .take_while(|&x| x != 0)
        .collect::<Vec<u16>>())
}

pub fn retrieve_command_choices(results: &IUnknown) -> Result<Vec<Words>> {
    let mut choices = Vec::new();

    let mut i = 0;
    while let Some(words) = retrieve_words(results, i)? {
        choices.push(words);
        i += 1;
    }

    Ok(choices)
}

fn retrieve_words(results: &IUnknown, choice: u32) -> Result<Option<Words>> {
    let results = query_interface::<ISRResGraph>(results)?;

    type Path = [u32; 512];
    let mut path: Path = [0u32; 512];
    let mut actual_path_size: u32 = 0;

    let rc = unsafe {
        results.best_path_word(
            choice,
            &mut path[0],
            mem::size_of::<Path>() as u32,
            &mut actual_path_size,
        )
    };
    if rc.0 == VALUE_OUT_OF_RANGE {
        return Ok(None);
    }
    rc.result()?;

    // bytes to number of elements
    let actual_path_size = actual_path_size / mem::size_of::<u32>() as u32;

    let mut word_node: SRRESWORDNODE = unsafe { mem::uninitialized() };
    let mut word: SRWORD = unsafe { mem::uninitialized() };
    let mut size_needed = 0u32;

    let mut words = Vec::new();
    for i in 0..actual_path_size {
        let rc = unsafe {
            results.get_word_node(
                path[i as usize],
                &mut word_node,
                &mut word,
                mem::size_of::<SRWORD>() as u32,
                &mut size_needed,
            )
        };
        rc.result()?;

        let info = WordInfo {
            text: string_from_slice(&word.buffer),
            rule: word_node.dwCFGParse,
            start_time: word_node.qwStartTime,
            end_time: word_node.qwEndTime,
        };

        words.push(info);
    }

    Ok(Some(words))
}

pub fn retrieve_selection_choices(results: &IUnknown, guid: GUID) -> Result<Vec<Selection>> {
    let mut choices = Vec::new();

    let mut i = 0;
    while let Some(selection) = retrieve_selection(results, guid, i)? {
        choices.push(selection);
        i += 1;
    }

    Ok(choices)
}

fn retrieve_selection(results: &IUnknown, guid: GUID, choice: u32) -> Result<Option<(u32, u32)>> {
    let results = query_interface::<IDgnSRResSelect>(results)?;

    let mut start = 0;
    let mut stop = 0;
    let mut word_number = 0;

    let rc = unsafe { results.get_info(guid, choice, &mut start, &mut stop, &mut word_number) };
    if rc.0 == VALUE_OUT_OF_RANGE {
        return Ok(None);
    }
    rc.result()?;

    Ok(Some((start, stop)))
}
