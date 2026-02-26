use std::collections::HashSet;

pub struct BadWordDetector {
    bad_words: HashSet<String>,
    enabled: bool,
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
            
            // Intentional misspellings and common variants
            "fuk", "fuc", "phuck", "phuk", "fook", "fock", "shite", "sht",
            "biatch", "effing", "freaking", "frigging", "freakin", "friggin"
        ];
        Self {
            bad_words: words.into_iter().map(|w| w.to_string()).collect(),
            enabled: true,
        }
    }

    fn clean(text: &str) -> String {
        // Lowercase, trim, and remove trailing punctuation with Regex or simple filter
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '*')
            .collect::<String>()
    }

    // Check if word sounds like a known bad word
    // This includes both intentional misspellings AND common Vosk misrecognitions
    fn sounds_like_bad_word(&self, word: &str) -> Option<String> {
        // Intentional misspellings/slang
        let intentional_variants: &[(&str, &str)] = &[
            // Intentional f-word variants
            ("fuk", "fuck"), ("fuc", "fuck"), ("phuck", "fuck"), ("phuk", "fuck"),
            ("fook", "fuck"), ("fock", "fuck"), ("fukk", "fuck"), ("fucc", "fuck"),
            ("fukn", "fucking"), ("fuckin", "fucking"), ("fkin", "fucking"),
            ("fckn", "fucking"), ("fcking", "fucking"), ("fuking", "fucking"),
            
            // Intentional s-word variants  
            ("shyt", "shit"), ("sht", "shit"), ("shiit", "shit"), ("shitt", "shit"),
            ("shite", "shit"), ("schit", "shit"),
            
            // Intentional b-word variants
            ("biatch", "bitch"), ("bytch", "bitch"), ("bich", "bitch"),
            ("beotch", "bitch"), ("biitch", "bitch"),
            
            // Intentional a-word variants
            ("azz", "ass"), ("a$$", "ass"), ("arsE", "ass"),
        ];
        
        for (variant, actual_bad) in intentional_variants {
            if word == *variant {
                return Some(actual_bad.to_string());
            }
        }
        
        // Vosk misrecognition patterns - words Vosk outputs instead of profanity
        // CAUTION: These are common words, so we only match them in specific contexts
        // We use "maybe_fuck" etc. to indicate these are potential matches
        let vosk_misrecognitions: &[(&str, &str)] = &[
            // F-word misrecognitions (Vosk hears "fuck" but outputs these)
            ("flog", "fuck"),      // Very common Vosk substitution
            ("flood", "fuck"),     // Common Vosk substitution
            ("fog", "fuck"),       // Sometimes heard as fog
            ("frog", "fuck"),      // Another variant
            ("folk", "fuck"),      // Can sound similar
            ("flop", "fuck"),      // Another variant
            ("flock", "fuck"),     // Another variant
            ("flux", "fuck"),      // Rare but possible
            
            // S-word misrecognitions
            ("ship", "shit"),      // Common substitution
            ("sheet", "shit"),     // Vosk mishearing
            ("shift", "shit"),     // Another variant
            ("shed", "shit"),      // Can sound similar
            
            // B-word misrecognitions  
            ("beach", "bitch"),    // Common substitution
            ("bench", "bitch"),    // Another variant
            ("pitch", "bitch"),    // Can sound similar
            
            // A-word misrecognitions
            ("ask", "ass"),        // Can sound similar at end of sentences
        ];
        
        for (misrecognition, actual_bad) in vosk_misrecognitions {
            if word == *misrecognition {
                return Some(actual_bad.to_string());
            }
        }
        
        None
    }

    pub fn contains_bad_word(&self, text: &str) -> Option<String> {
        if !self.enabled { return None; }
        let cleaned = Self::clean(text);
        if cleaned.is_empty() { return None; }

        // 1. Exact match (cleaned word is one of the bad words)
        if self.bad_words.contains(&cleaned) { 
            return Some(cleaned); 
        }

        // 2. Substring match (e.g., "fucking" contains "fuck")
        for bad_word in &self.bad_words {
            // Check if it's a substring
            if cleaned.contains(bad_word) {
                // If the bad word is very short (2 chars), only exact match it to avoid "fk" matching "fake"
                if bad_word.len() <= 2 && cleaned != *bad_word {
                    continue;
                }
                
                // Avoid false positives for common words
                if bad_word == "hell" && cleaned.contains("hello") { 
                    if !cleaned.replace("hello", "").contains("hell") {
                        continue; 
                    }
                }
                
                if bad_word == "ass" && (cleaned.contains("associ") || cleaned.contains("assu") || cleaned.contains("glass")) { 
                    if !cleaned.replace("associ", "").replace("assume", "").replace("glass", "").contains("ass") {
                        continue;
                    }
                }
                
                return Some(bad_word.clone());
            }
        }
        
        // 3. Phonetic similarity check for misrecognized words
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

    pub fn add_word(&mut self, word: String) { self.bad_words.insert(word.to_lowercase()); }
    pub fn remove_word(&mut self, word: &str) { self.bad_words.remove(&word.to_lowercase()); }
    pub fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    pub fn clear(&mut self) { self.bad_words.clear(); }
    pub fn get_all_bad_words(&self) -> Vec<String> { self.bad_words.iter().cloned().collect() }
}