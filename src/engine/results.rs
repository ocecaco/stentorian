use super::events::GrammarEvent;
use crate::dragon::{SRRESWORDNODE, SRWORD, VOICEPARTOFSPEECH};
use crate::errors::Result;
use crate::interfaces::{IDgnSRResSelect, ISRResGraph};
use components::{Cast, GUID};
use serde::Serialize;
use std::mem;

const VALUE_OUT_OF_RANGE: u32 = 0x8000_FFFF;
const NOT_A_SELECT_RESULT: u32 = 0x8004_1019;

pub type DictationGrammarEvent = GrammarEvent<Vec<Words>>;
pub type CatchallGrammarEvent = GrammarEvent<Vec<Words>>;
pub type CommandGrammarEvent = GrammarEvent<Words>;
pub type Words = Vec<WordInfo>;

#[derive(Debug, Serialize)]
pub struct WordInfo {
    pub text: String,
    pub start_time: u64,
    pub end_time: u64,
}

pub type Selection = (Words, u32, u32);
pub type SelectGrammarEvent = GrammarEvent<Vec<Selection>>;

fn string_from_slice(s: &[u16]) -> String {
    String::from_utf16_lossy(
        &s.iter()
            .cloned()
            .take_while(|&x| x != 0)
            .collect::<Vec<u16>>(),
    )
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

pub fn retrieve_words(results: &ISRResGraph, choice: u32) -> Result<Option<Words>> {
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

    let mut word_node: SRRESWORDNODE = SRRESWORDNODE {
        dwNextWordNode: 0,
        dwUpAlternateWordNode: 0,
        dwDownAlternateWordNode: 0,
        dwPreviousWordNode: 0,
        dwPhonemeNode: 0,
        qwStartTime: 0,
        qwEndTime: 0,
        dwWordScore: 0,
        wVolume: 0,
        wPitch: 0,
        pos: VOICEPARTOFSPEECH::VPS_UNKNOWN,
        dwCFGParse: 0,
        dwCue: 0,
    };

    let mut word: SRWORD = SRWORD {
        size: 0,
        word_number: 0,
        buffer: [0u16; 128],
    };

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
            start_time: word_node.qwStartTime,
            end_time: word_node.qwEndTime,
        };

        words.push(info);
    }

    Ok(Some(words))
}

pub fn retrieve_selection_choices(
    select_results: &IDgnSRResSelect,
    guid: GUID,
) -> Result<Vec<Selection>> {
    let graph_results = select_results.cast()?;
    let mut choices = Vec::new();

    let mut i = 0;
    while let Some(selection) = retrieve_selection(select_results, guid, i)? {
        if let Some((a, b)) = selection {
            let words = retrieve_words(&graph_results, i)?.unwrap();
            choices.push((words, a, b));
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
