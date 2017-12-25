use super::events::GrammarEvent;
use components::GUID;
use dragon::{SRRESWORDNODE, SRWORD};
use errors::Result;
use interfaces::{IDgnSRResSelect, ISRResGraph};
use std::mem;

const VALUE_OUT_OF_RANGE: u32 = 0x8000_FFFF;
const NOT_A_SELECT_RESULT: u32 = 0x8004_1019;

pub type CommandGrammarEvent = GrammarEvent<Vec<Words>>;
pub type Words = Vec<WordInfo>;

#[derive(Debug, Serialize)]
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

pub fn retrieve_command_choices(results: &ISRResGraph) -> Result<Vec<Words>> {
    let mut choices = Vec::new();

    let mut i = 0;
    while let Some(words) = retrieve_words(results, i)? {
        choices.push(words);
        i += 1;
    }

    Ok(choices)
}

fn retrieve_words(results: &ISRResGraph, choice: u32) -> Result<Option<Words>> {
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

pub fn retrieve_selection_choices(results: &IDgnSRResSelect, guid: GUID) -> Result<Vec<Selection>> {
    let mut choices = Vec::new();

    let mut i = 0;
    while let Some(selection) = retrieve_selection(results, guid, i)? {
        if let Some(range) = selection {
            choices.push(range);
        }
        i += 1;
    }

    Ok(choices)
}

fn retrieve_selection(
    results: &IDgnSRResSelect,
    guid: GUID,
    choice: u32,
) -> Result<Option<Option<(u32, u32)>>> {
    let mut start = 0;
    let mut stop = 0;
    let mut word_number = 0;

    let rc = unsafe { results.get_info(guid, choice, &mut start, &mut stop, &mut word_number) };
    if rc.0 == VALUE_OUT_OF_RANGE {
        return Ok(None);
    }
    if rc.0 == NOT_A_SELECT_RESULT {
        return Ok(Some(None));
    }
    rc.result()?;

    Ok(Some(Some((start, stop))))
}
