//! Bad Word Detector with Vosk misrecognition mapping

use std::collections::HashSet;

pub struct BadWordDetector {
    bad_words: HashSet<String>,
}

impl BadWordDetector {
    pub fn new() -> Self {
        let words = vec![
            // Core bad words
            "damn", "hell", "crap", "ass", "fuck", "shit", "dick", "pussy", 
            "bitch", "bastard", "idiot", "stupid", "jerk", "weapon", "kill",
            "fucking", "shitting", "bitches", "fck", "fk", "sh*t", "f*ck",
            "motherfucker", "fucker", "asshole", "bitchy", "dumbass",
            "f*cking", "sh*tting", "fcuk", "shyt", "dammit", "hellish",
            "dumb", "shut up", "bloody", "cunt", "nigger", "slut", "whore",
            
            // Intentional misspellings
            "fuk", "fuc", "phuck", "phuk", "fook", "fock", "shite", "sht",
            "biatch", "effing", "freaking", "frigging", "freakin", "friggin"
        ];
        Self {
            bad_words: words.into_iter().map(|w| w.to_string()).collect(),
        }
    }

    fn clean(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '*')
            .collect()
    }

    /// Check if word sounds like a bad word (including Vosk misrecognitions)
    fn sounds_like_bad_word(&self, word: &str) -> Option<String> {
        // Intentional misspellings
        let intentional_variants: &[(&str, &str)] = &[
            ("fuk", "fuck"), ("fuc", "fuck"), ("phuck", "fuck"), ("phuk", "fuck"),
            ("fook", "fuck"), ("fock", "fuck"), ("fukk", "fuck"), ("fucc", "fuck"),
            ("fukn", "fucking"), ("fuckin", "fucking"), ("fkin", "fucking"),
            ("shyt", "shit"), ("sht", "shit"), ("shiit", "shit"), ("shite", "shit"),
            ("biatch", "bitch"), ("bytch", "bitch"), ("bich", "bitch"),
            ("azz", "ass"), ("a$$", "ass"),
        ];
        
        for (variant, actual_bad) in intentional_variants {
            if word == *variant {
                return Some(actual_bad.to_string());
            }
        }
        
        // Vosk misrecognition patterns
        let vosk_misrecognitions: &[(&str, &str)] = &[
            // F-word misrecognitions
            ("flog", "fuck"), ("flood", "fuck"), ("fog", "fuck"), ("frog", "fuck"),
            ("folk", "fuck"), ("flop", "fuck"), ("flock", "fuck"), ("flux", "fuck"),
            
            // S-word misrecognitions
            ("ship", "shit"), ("sheet", "shit"), ("shift", "shit"), ("shed", "shit"),
            
            // B-word misrecognitions
            ("beach", "bitch"), ("bench", "bitch"), ("pitch", "bitch"),
            
            // A-word misrecognitions
            ("ask", "ass"),
        ];
        
        for (misrecognition, actual_bad) in vosk_misrecognitions {
            if word == *misrecognition {
                return Some(actual_bad.to_string());
            }
        }
        
        None
    }

    pub fn contains_bad_word(&self, text: &str) -> Option<String> {
        let cleaned = Self::clean(text);
        if cleaned.is_empty() { return None; }

        // 1. Exact match
        if self.bad_words.contains(&cleaned) { 
            return Some(cleaned); 
        }

        // 2. Substring match
        for bad_word in &self.bad_words {
            if cleaned.contains(bad_word) {
                if bad_word.len() <= 2 && cleaned != *bad_word {
                    continue;
                }
                
                // Avoid false positives
                if bad_word == "hell" && cleaned.contains("hello") { 
                    if !cleaned.replace("hello", "").contains("hell") {
                        continue; 
                    }
                }
                
                if bad_word == "ass" && (cleaned.contains("associ") || cleaned.contains("assu") || cleaned.contains("glass")) { 
                    continue;
                }
                
                return Some(bad_word.clone());
            }
        }
        
        // 3. Phonetic similarity check
        if let Some(match_word) = self.sounds_like_bad_word(&cleaned) {
            return Some(match_word);
        }
        
        None
    }

    pub fn detect_all_bad_words(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter_map(|w| self.contains_bad_word(w))
            .collect()
    }
}
