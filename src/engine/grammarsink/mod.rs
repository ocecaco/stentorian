pub mod interfaces;

use std::sync::mpsc::Sender;
use self::interfaces::*;
use interfaces::*;
use components::*;
use components::comptr::ComPtr;
use components::refcount::*;
use std::boxed::Box;
use std::mem;
use dragon::*;
use super::{GrammarEvent, GrammarSinkFlags};
use super::events::{EventSender, ConvertSender};

use std::os::raw::c_void;

#[repr(C)]
pub struct GrammarSink {
    vtable1: &'static ISRGramNotifySinkVtable,
    vtable2: &'static IDgnGetSinkFlagsVtable,
    ref_count: RefCount,
    flags: GrammarSinkFlags,
    events: Box<EventSender<GrammarEvent> + Sync>,
}

impl GrammarSink {
    pub fn create<T>(flags: GrammarSinkFlags, sender: Sender<T>) -> ComPtr<IUnknown>
        where T: From<GrammarEvent> + Send + 'static
    {
        fn ensure_sync<T: Sync>(_: &T) {}

        let result = GrammarSink {
            vtable1: &v1::VTABLE,
            vtable2: &v2::VTABLE,
            ref_count: RefCount::new(1),
            flags: flags,
            events: Box::new(ConvertSender::new(sender)),
        };

        ensure_sync(&result);

        let raw = Box::into_raw(Box::new(result)) as RawComPtr;
        unsafe { raw_to_comptr(raw, true) }
    }

    unsafe fn query_interface(&self, iid: *const IID, v: *mut RawComPtr) -> HRESULT {
        query_interface! {
            self, iid, v,
            IUnknown => vtable1,
            ISRGramNotifySink => vtable1,
            IDgnGetSinkFlags => vtable2
        }

        self.ref_count.up();
        HRESULT(0)
    }

    unsafe fn add_ref(&self) -> u32 {
        self.ref_count.up()
    }

    unsafe fn release(&self) -> u32 {
        let result = self.ref_count.down();

        if result == 0 {
            Box::from_raw(self as *const _ as *mut GrammarSink);
        }

        result
    }

    fn bookmark(&self, x: u32) -> HRESULT {
        self.events.send(GrammarEvent::Bookmark);
        HRESULT(0)
    }
    fn paused(&self) -> HRESULT {
        self.events.send(GrammarEvent::Paused);
        HRESULT(0)
    }
    unsafe fn phrase_finish(&self,
                            a: u32,
                            b: u64,
                            c: u64,
                            phrase: *const c_void,
                            results: RawComPtr)
                            -> HRESULT {
        let results = raw_to_comptr::<IUnknown>(results, false);
        let results = query_interface::<ISRResGraph>(&results).unwrap();

        type Path = [u32; 512];
        let mut path: Path = [0u32; 512];
        let mut actual_path_size: u32 = 0;

        let rc = results.best_path_word(0,
                                        &mut path[0],
                                        mem::size_of::<Path>() as u32,
                                        &mut actual_path_size);
        assert_eq!(rc.0, 0);

        // bytes to number of elements
        let actual_path_size = actual_path_size / mem::size_of::<u32>() as u32;

        let mut word_node: SRRESWORDNODE = mem::uninitialized();
        let mut word: SRWORD = mem::uninitialized();
        let mut size_needed = 0u32;

        let mut words = Vec::new();
        for i in 0..actual_path_size {
            let rc = results.get_word_node(path[i as usize],
                                           &mut word_node,
                                           &mut word,
                                           mem::size_of::<SRWORD>() as u32,
                                           &mut size_needed);
            assert_eq!(rc.0, 0);

            words.push((string_from_slice(&word.buffer), word_node.dwCFGParse));
        }
        let words = words.into_boxed_slice();

        self.events.send(GrammarEvent::PhraseFinish(words));

        HRESULT(0)
    }
    fn phrase_hypothesis(&self,
                         a: u32,
                         b: u64,
                         c: u64,
                         phrase: *const c_void,
                         results: RawComPtr)
                         -> HRESULT {
        self.events.send(GrammarEvent::PhraseHypothesis);
        HRESULT(0)
    }
    fn phrase_start(&self, a: u64) -> HRESULT {
        self.events.send(GrammarEvent::PhraseStart);
        HRESULT(0)
    }
    fn reevaluate(&self, a: RawComPtr) -> HRESULT {
        self.events.send(GrammarEvent::Reevaluate);
        HRESULT(0)
    }
    fn training(&self, a: u32) -> HRESULT {
        self.events.send(GrammarEvent::Training);
        HRESULT(0)
    }
    fn unarchive(&self, a: RawComPtr) -> HRESULT {
        self.events.send(GrammarEvent::Unarchive);
        HRESULT(0)
    }

    unsafe fn sink_flags_get(&self, flags: *mut u32) -> HRESULT {
        *flags = self.flags.bits();
        HRESULT(0)
    }
}

fn string_from_slice(s: &[u16]) -> String {
    String::from_utf16_lossy(&s.iter()
                                  .cloned()
                                  .take_while(|&x| x != 0)
                                  .collect::<Vec<u16>>())
}

coclass! {
    GrammarSink {
        mod v1 in vtable1 {
            vtable_name: VTABLE,

            interface ISRGramNotifySink {
                vtable: ISRGramNotifySinkVtable,
                interface IUnknown {
                    vtable: IUnknownVtable,
                    fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                    fn add_ref() -> ULONG;
                    fn release() -> ULONG;
                },
                fn bookmark(x: u32) -> HRESULT;
                fn paused() -> HRESULT;
                fn phrase_finish(a: u32,
                                 b: u64,
                                 c: u64,
                                 phrase: *const c_void,
                                 results: RawComPtr) -> HRESULT;
                fn phrase_hypothesis(a: u32,
                                     b: u64,
                                     c: u64,
                                     phrase: *const c_void,
                                     results: RawComPtr) -> HRESULT;
                fn phrase_start(a: u64) -> HRESULT;
                fn reevaluate(a: RawComPtr) -> HRESULT;
                fn training(a: u32) -> HRESULT;
                fn unarchive(a: RawComPtr) -> HRESULT;
            }
        }

        mod v2 in vtable2 {
            vtable_name: VTABLE,

            interface IDgnGetSinkFlags {
                vtable: IDgnGetSinkFlagsVtable,
                interface IUnknown {
                    vtable: IUnknownVtable,
                    fn query_interface(iid: *const IID, v: *mut RawComPtr) -> HRESULT;
                    fn add_ref() -> ULONG;
                    fn release() -> ULONG;
                },
                fn sink_flags_get(flags: *mut u32) -> HRESULT;
            }
        }
    }
}
