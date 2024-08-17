use core::ops::Index;
use std::fmt::Display;



#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NameTableId(u32);


impl Display for NameTableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


pub struct NameTable {
    names: Vec<String>,
}

impl NameTable {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &str) -> NameTableId {
        // Naive implementation
        let it = self.names.iter().enumerate().find(|(_, val)| *val == name);
        match it {
            Some((idx, _)) => NameTableId(idx as u32),
            None => {
                let idx = self.names.len() as u32;
                self.names.push(name.to_string());
                NameTableId(idx)
            },
        }
    }


    pub fn get_name(&self, index: NameTableId) -> &str {
        &self.names[index.0 as usize]
    }
    
}

impl Index<NameTableId> for NameTable {
    type Output = str;

    fn index(&self, index: NameTableId) -> &Self::Output {
        self.get_name(index)
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PunctuationId(u32);

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