use components::*;

type LANGID = u16;

const LANG_LEN: usize = 64;
const SVFN_LEN: usize = 262;
const SRMI_NAMELEN: usize = SVFN_LEN;

#[allow(non_snake_case)]
#[repr(C)]
pub struct LANGUAGE {
    LanguageID: LANGID,
    szDialect: [u16; LANG_LEN],
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct SRMODEINFO {
    gEngineID: GUID,
    szMfgName: [u16; SRMI_NAMELEN],
    pub szProductName: [u16; SRMI_NAMELEN],
    gModeID: GUID,
    szModeName: [u16; SRMI_NAMELEN],
    language: LANGUAGE,
    dwSequencing: u32,
    dwMaxWordsVocab: u32,
    dwMaxWordsState: u32,
    dwGrammars: u32,
    dwFeatures: u32,
    dwInterfaces: u32,
    dwEngineFeatures: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum SRGRMFMT {
    SRGRMFMT_CFG = 0x0000,
    SRGRMFMT_LIMITEDDOMAIN = 0x0001,
    SRGRMFMT_DICTATION = 0x0002,
    SRGRMFMT_CFGNATIVE = 0x8000,
    SRGRMFMT_LIMITEDDOMAINNATIVE = 0x8001,
    SRGRMFMT_DICTATIONNATIVE = 0x8002,
    SRGRMFMT_DRAGONNATIVE1 = 0x8101,
    SRGRMFMT_DRAGONNATIVE2 = 0x8102,
    SRGRMFMT_DRAGONNATIVE3 = 0x8103,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SDATA {
    pub data: *const u8,
    pub size: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum VOICEPARTOFSPEECH {
    VPS_UNKNOWN = 0,
    VPS_NOUN = 1,
    VPS_VERB = 2,
    VPS_ADVERB = 3,
    VPS_ADJECTIVE = 4,
    VPS_PROPERNOUN = 5,
    VPS_PRONOUN = 6,
    VPS_CONJUNCTION = 7,
    VPS_CARDINAL = 8,
    VPS_ORDINAL = 9,
    VPS_DETERMINER = 10,
    VPS_QUANTIFIER = 11,
    VPS_PUNCTUATION = 12,
    VPS_CONTRACTION = 13,
    VPS_INTERJECTION = 14,
    VPS_ABBREVIATION = 15,
    VPS_PREPOSITION = 16,
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SRRESWORDNODE {
    dwNextWordNode: u32,
    dwUpAlternateWordNode: u32,
    dwDownAlternateWordNode: u32,
    dwPreviousWordNode: u32,
    dwPhonemeNode: u32,
    qwStartTime: u64,
    qwEndTime: u64,
    dwWordScore: u32,
    wVolume: u16,
    wPitch: u16,
    pos: VOICEPARTOFSPEECH,
    pub dwCFGParse: u32,
    dwCue: u32,
}

#[repr(C)]
pub struct SRWORD {
    pub size: u32,
    pub word_number: u32,
    pub buffer: [u16; 128],
}

define_guid!(pub CLSID_DgnDictate = 0xdd100001,
             0x6205,
             0x11cf,
             0xae,
             0x61,
             0x00,
             0x00,
             0xe8,
             0xa2,
             0x86,
             0x47);

define_guid!(pub CLSID_DgnSite = 0xdd100006,
             0x6205,
             0x11cf,
             0xae,
             0x61,
             0x00,
             0x00,
             0xe8,
             0xa2,
             0x86,
             0x47);
