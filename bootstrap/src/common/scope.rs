use std::{
    hash::Hash,
    fmt,
};


#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ScopeSegment {
    pub name:   String,
    pub params: Vec<String>
}

impl ScopeSegment {
    fn new(name: String) -> Self {
        Self {
            name,
            params: Vec::new(),
        }
    }

    fn new_with_params(name: String, params: Vec<String>) -> Self {
        Self {
            name,
            params,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Scope {
    segments: Vec<ScopeSegment>,
}

#[allow(unused)]
impl Scope {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push(&mut self, name: String) {
        self.segments.push(ScopeSegment::new(name));
    }

    pub fn push_with_params(&mut self, name: String, params: Vec<String>) {
        self.segments.push(ScopeSegment::new_with_params(name, params));
    }

    pub fn push_segment(&mut self, segment: ScopeSegment) {
        self.segments.push(segment);
    }

    pub fn extend(&mut self, extension: &Scope) {
        for segment in &extension.segments {
            self.push_segment(segment.clone());
        }
    }

    pub fn pop(&mut self) {
        self.segments.pop();
    }

    pub fn segments(&self) -> &Vec<ScopeSegment> {
        &self.segments
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }
    
    pub fn parent(&self) -> Scope {
        if self.segments.len() <= 1 {
            return Scope::new();
        }

        let mut parent = Scope::new();
        for segment in &self.segments[..self.segments.len() - 1] {
            parent.segments.push(segment.clone());
        }
        parent
    }

    // Get the path without it's root
    pub fn sub_path(&self) -> Scope {
        if self.segments.len() <= 1 {
            return Scope::new();
        }

        let mut sub_path = Scope::new();
        for segment in &self.segments[1..] {
            sub_path.segments.push(segment.clone());
        }
        sub_path
    }

    pub fn root(&self) -> Option<&ScopeSegment> {
        self.segments.first()
    }

    pub fn last(&self) -> Option<&ScopeSegment> {
        self.segments.last()
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, segment) in self.segments.iter().enumerate() {
            if idx != 0 {
                write!(f, ".")?;
            }

            write!(f, "{}", &segment.name)?;
            if !segment.params.is_empty() {
                write!(f, "(")?;

                for (idx, param) in segment.params.iter().enumerate() {
                    if idx != 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", param)?;
                }

                write!(f, ")")?;
            }
        }
        Ok(())
    }
}