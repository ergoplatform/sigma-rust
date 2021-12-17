//! Word lists for mnemonic generation

use std::{fmt, str::FromStr};

/// Supported languages for mnemonic phrases
#[derive(Debug, Clone, Copy)]
pub enum Language {
    /// Simplified Chinese wordlist
    ChineseSimplified,
    /// Traditional Chinese wordlist
    ChineseTraditional,
    /// English wordlist
    English,
    /// French wordlist
    French,
    /// Italian wordlist
    Italian,
    /// Japanese wordlist
    Japanese,
    /// Korean wordlist
    Korean,
    /// Spanish wordlist
    Spanish,
}

/// Language error relating to mnemonic generation
#[derive(Debug, Clone, Copy)]
pub enum LanguageError {
    /// Unsupported language when trying to parse `Language` from a string
    InvalidStr,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lang = match self {
            Language::ChineseSimplified => "chinese_simplified",
            Language::ChineseTraditional => "chinese_traditional",
            Language::English => "english",
            Language::French => "french",
            Language::Italian => "italian",
            Language::Japanese => "japanese",
            Language::Korean => "korean",
            Language::Spanish => "spanish",
        };
        write!(f, "{}", lang)
    }
}

impl FromStr for Language {
    type Err = LanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "chinese_simplified" => Ok(Language::ChineseSimplified),
            "chinese_traditional" => Ok(Language::ChineseTraditional),
            "english" => Ok(Language::English),
            "french" => Ok(Language::French),
            "italian" => Ok(Language::Italian),
            "japanese" => Ok(Language::Japanese),
            "korean" => Ok(Language::Korean),
            "spanish" => Ok(Language::Spanish),
            _ => Err(LanguageError::InvalidStr),
        }
    }
}

static CHINESE_SIMPLIFIED: &str = include_str!("./wordlist/chinese_simplified.txt");
static CHINESE_TRADITIONAL: &str = include_str!("./wordlist/chinese_traditional.txt");
static ENGLISH: &str = include_str!("./wordlist/english.txt");
static FRENCH: &str = include_str!("./wordlist/french.txt");
static ITALIAN: &str = include_str!("./wordlist/italian.txt");
static JAPANESE: &str = include_str!("./wordlist/japanese.txt");
static KOREAN: &str = include_str!("./wordlist/korean.txt");
static SPANISH: &str = include_str!("./wordlist/spanish.txt");

/// Wordlists
pub struct WordList(pub Language);

impl WordList {
    /// Return a list of words used for mnemonics
    pub fn words(&self) -> Vec<&'static str> {
        match self.0 {
            Language::ChineseSimplified => CHINESE_SIMPLIFIED,
            Language::ChineseTraditional => CHINESE_TRADITIONAL,
            Language::English => ENGLISH,
            Language::French => FRENCH,
            Language::Italian => ITALIAN,
            Language::Japanese => JAPANESE,
            Language::Korean => KOREAN,
            Language::Spanish => SPANISH,
        }
        .lines()
        .collect()
    }

    /// Returns the word separator for mnemonics
    pub fn delimiter(&self) -> &'static str {
        match self.0 {
            Language::Japanese => "　", // \u3000
            Language::ChineseSimplified
            | Language::ChineseTraditional
            | Language::English
            | Language::French
            | Language::Italian
            | Language::Korean
            | Language::Spanish => " ",
        }
    }
}
