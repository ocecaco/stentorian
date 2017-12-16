use std::mem;
use dragon::*;
use interfaces::*;
use components::*;
use errors::*;

fn string_from_slice(s: &[u16]) -> String {
    String::from_utf16_lossy(&s.iter()
                                  .cloned()
                                  .take_while(|&x| x != 0)
                                  .collect::<Vec<u16>>())
}

pub fn retrieve_words(results: &IUnknown) -> Result<Box<[(String, u32)]>> {
    let results = query_interface::<ISRResGraph>(&results)?;

    type Path = [u32; 512];
    let mut path: Path = [0u32; 512];
    let mut actual_path_size: u32 = 0;

    let rc = unsafe {
        results.best_path_word(0,
                               &mut path[0],
                               mem::size_of::<Path>() as u32,
                               &mut actual_path_size)
    };
    rc.result()?;

    // bytes to number of elements
    let actual_path_size = actual_path_size / mem::size_of::<u32>() as u32;

    let mut word_node: SRRESWORDNODE = unsafe { mem::uninitialized() };
    let mut word: SRWORD = unsafe { mem::uninitialized() };
    let mut size_needed = 0u32;

    let mut words = Vec::new();
    for i in 0..actual_path_size {
        let rc = unsafe {
            results.get_word_node(path[i as usize],
                                  &mut word_node,
                                  &mut word,
                                  mem::size_of::<SRWORD>() as u32,
                                  &mut size_needed)
        };
        rc.result()?;

        words.push((string_from_slice(&word.buffer), word_node.dwCFGParse));
    }

    Ok(words.into_boxed_slice())
}
