use std::{fmt, ops::Index};









#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub file_id:     u32,
    pub char_offset: u64,
    pub byte_offset: u64,
    pub char_len:    u64,
    pub byte_len:    u64,
    pub row:         u32,
    pub row_end:    u32,
    pub column:      u32,
    pub column_end: u32,
}

impl Span {
}



#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpanId(usize);

impl SpanId {
    pub const INVALID: SpanId = SpanId(usize::MAX);
}

pub struct SpanRegistry {
    pub files: Vec<String>,
    pub spans: Vec<Span>,
}

#[allow(unused)]
impl SpanRegistry {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            spans: Vec::new(),
        }
    }

    pub fn add_span(&mut self, file: &str, mut span: Span) -> SpanId {
        let file_id = match self.files.iter().enumerate().find(|(_, path)| *path == file).map(|(id, _)| id) {
            Some(file_id) => file_id as u32,
            None => {
                let file_id = self.files.len() as u32;
                self.files.push(file.to_string());
                file_id
            },
        };

        span.file_id = file_id;
        self.add_span_(span)
    }

    fn add_span_(&mut self, span: Span) -> SpanId {
        let id = SpanId(self.spans.len() as usize);
        self.spans.push(span);
        id
    }

    pub fn combine_spans(&mut self, begin: SpanId, end: SpanId) -> SpanId {
        if begin == end {
            return begin;
        }
        if begin == SpanId::INVALID || end == SpanId::INVALID {
            return SpanId::INVALID;
        }

        let begin = self.spans[begin.0];
        let end = self.spans[end.0];

        if begin.file_id != end.file_id {
            return SpanId::INVALID;
        }

        let mut span = begin;
        span.char_len = end.char_offset + end.char_len - begin.char_offset;
        span.byte_len = end.byte_offset + end.byte_len - begin.byte_offset;
        span.row_end = end.row_end;
        span.column_end = end.column_end;

        self.add_span_(span)
    }

    pub fn get_file(&self, file_id: u32) -> &str {
        &self.files[file_id as usize]
    }

    pub fn get_file_from_span_id(&self, span: SpanId) -> Option<&str> {
        if span.0 >= self.spans.len() {
            None
        } else {
            let file_id = self.spans[span.0].file_id;
            Some(&self.files[file_id as usize])
        }
    }
}

impl Index<SpanId> for SpanRegistry {
    type Output = Span;

    fn index(&self, index: SpanId) -> &Self::Output {
        &self.spans[index.0]
    }
}

pub struct FormatSpanLoc<'a> {
    pub registry: &'a SpanRegistry,
    pub span:     SpanId
}

impl fmt::Display for FormatSpanLoc<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let span = &self.registry.spans[self.span.0];
        let file = &self.registry.files[span.file_id as usize];
        write!(f, "{file}:{}:{}", span.row, span.column)
    }
}


pub struct FormatSpan<'a> {
    pub registry: &'a SpanRegistry,
    pub span:     SpanId
}

impl fmt::Display for FormatSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let span = &self.registry.spans[self.span.0];
        let file = &self.registry.files[span.file_id as usize];
        write!(f, "{file}, {}:{}->{}:{} (chars: {}-{}, bytes: {}-{})",
            span.row,
            span.column,
            span.row_end,
            span.column_end,
            span.char_offset,
            span.char_offset + span.char_len,
            span.byte_offset,
            span.byte_offset + span.byte_len,
        )
    }
}
