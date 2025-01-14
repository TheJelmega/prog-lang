use core::ops::Index;
use std::fmt::Display;

use super::WeakKeyword;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
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