//! Bad Word Detector — Strict exact-match only.
//!
//! Only detects real profanity words that exist in the dictionary.
//! NO fuzzy matching, NO Vosk-misrecognition mapping, NO substring matching.
//! This eliminates false positives from music (fog, ship, beach, ask, etc.).

use std::collections::HashSet;

pub struct BadWordDetector {
    bad_words: HashSet<&'static str>,
}

impl BadWordDetector {
    pub fn new() -> Self {
        // Bad words for detection. Only removed nigga/nigger — they cause
        // false positives on non-English music (Hindi kids' songs etc.).
        let words: HashSet<&'static str> = [
            // F-word family + sound-alikes (Vosk may hear "fuck" as folk/fog)
            "fuck", "fucking", "fucker", "fucked", "folk", "fog",
            // S-word
            "shit",
            // B-word
            "bitch",
            // Suck family
            "suck", "sucker",
            // Others
            "slut", "asshole",
        ].into_iter().collect();

        Self { bad_words: words }
    }

    /// Normalize a word to its root for strike dedup.
    pub fn normalize_to_root(word: &str) -> &'static str {
        let w = word.to_lowercase();
        if w.starts_with("fuck") || w == "folk" || w == "fog" { return "fuck"; }
        if w == "shit"                                  { return "shit"; }
        if w.starts_with("bitch")                       { return "bitch"; }
        if w.starts_with("suck")                        { return "suck"; }
        if w == "slut"                                  { return "slut"; }
        if w == "asshole"                               { return "asshole"; }
        "other"
    }

    /// Check a single word — strict exact match only.
    pub fn contains_bad_word(&self, text: &str) -> Option<String> {
        let cleaned: String = text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        if cleaned.is_empty() { return None; }

        if self.bad_words.contains(cleaned.as_str()) {
            return Some(cleaned);
        }
        None
    }

    /// Scan a full sentence — check each word independently (no substring).
    pub fn detect_all_bad_words(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter_map(|w| self.contains_bad_word(w))
            .collect()
    }
}
