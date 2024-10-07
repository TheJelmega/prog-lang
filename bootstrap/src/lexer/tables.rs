use core::ops::Index;
use std::fmt::Display;

use super::WeakKeyword;



#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NameId(u32);

impl NameId {
    pub const INVALID: Self = NameId(u32::MAX);
}

impl Display for NameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


pub struct NameTable {
    names: Vec<String>,
}

impl NameTable {
    pub fn new() -> Self {
        // Setup names for weak keywords
        let mut names = Vec::with_capacity(WeakKeyword::WEAK_KEYWORD_NAMES.len());
        for kw in WeakKeyword::WEAK_KEYWORD_NAMES {
            names.push(kw.to_string());
        }
        
        Self {
            names,
        }
    }

    pub fn add(&mut self, name: &str) -> NameId {
        // Naive implementation
        let it = self.names.iter().enumerate().find(|(_, val)| *val == name);
        match it {
            Some((idx, _)) => NameId(idx as u32),
            None => {
                let idx = self.names.len() as u32;
                self.names.push(name.to_string());
                NameId(idx)
            },
        }
    }

    pub fn get_id_for_weak_kw(&self, kw: WeakKeyword) -> NameId {
        NameId(kw as u32)
    }

    pub fn get_name(&self, index: NameId) -> &str {
        &self.names[index.0 as usize]
    }
    
}

impl Index<NameId> for NameTable {
    type Output = str;

    fn index(&self, index: NameId) -> &Self::Output {
        self.get_name(index)
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PunctuationId(u32);

impl PunctuationId {
    pub const INVALID: Self = PunctuationId(u32::MAX);
}

impl Display for PunctuationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


pub struct PuncutationTable {
    punctuation: Vec<String>,
}

impl PuncutationTable {
    pub fn new() -> Self {
        Self { punctuation: Vec::new() }
    }

    pub fn add(&mut self, punct: &str) -> PunctuationId {
        // Naive implementation
        let it = self.punctuation.iter().enumerate().find(|(_, val)| *val == punct);
        match it {
            Some((idx, _)) => PunctuationId(idx as u32),
            None => {
                let idx = self.punctuation.len() as u32;
                self.punctuation.push(punct.to_string());
                PunctuationId(idx)
            }
        }
    }

    pub fn get(&self, index: PunctuationId) -> &str {
        &self.punctuation[index.0 as usize]
    }
}

impl Index<PunctuationId> for PuncutationTable {
    type Output = str;

    fn index(&self, index: PunctuationId) -> &Self::Output {
        self.get(index)
    }
}