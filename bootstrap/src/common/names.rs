use std::{fmt, ops::Index};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NameId(u32);

impl NameId {
    pub const INVALID: Self = NameId(u32::MAX);
}

impl fmt::Display for NameId {
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

    // When no name with the given path exists, `NameId::INVALID` is returned, which will never match any parsed name
    pub fn get_id_for_str(&self, name: &str) -> NameId {
        self.names.iter()
            .enumerate()
            .find(|(_, val)| *val == name)
            .map_or(NameId::INVALID, |(id, _)| NameId(id as u32))
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
