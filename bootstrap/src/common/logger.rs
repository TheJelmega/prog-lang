#![allow(unused)]

use std::{
    io::Write,
    fmt,
};

pub struct Logger {
    
}

impl Logger {
    pub fn new() -> Self {
        Self {}
    }

    pub fn log_fmt(&self, args: fmt::Arguments) {
        let stdout = std::io::stdout();
        _ = stdout.lock().write_fmt(args);
    }

    pub fn log(&self, s: &str) {
        self.log_fmt(format_args!("{s}"));
    }
    
    pub fn logln(&self, s: &str) {
        self.log_fmt(format_args!("{s}\n"));
    }
}

pub struct IndentLogger {
    logger:       Logger,
    empty_indent: &'static str,
    full_indent:  &'static str,
    cur_indent:   &'static str,
    indents:      Vec<bool>,
}

impl IndentLogger {
    pub fn new(empty_indent: &'static str, full_indent: &'static str, cur_indent: &'static str) -> Self {
        Self {
            logger: Logger::new(),
            empty_indent,
            full_indent,
            cur_indent,
            indents: vec![true],
        }
    }
    
    pub fn write_prefix(&self) {
        for bit in &self.indents[..self.indents.len() - 1]   {
            if *bit {
                self.log(self.full_indent);
            } else {
                self.log(self.empty_indent);
            }
        }
        self.log(self.cur_indent)
    }

    pub fn log_fmt(&self, args: fmt::Arguments) {
        self.logger.log_fmt(args);
    }

    pub fn prefixed_log_fmt(&self, args: fmt::Arguments) {
        self.write_prefix();
        self.log_fmt(args);
    }

    pub fn log(&self, s: &str) {
        self.log_fmt(format_args!("{s}"));
    }
    
    pub fn prefixed_log(&self, s: &str) {
        self.write_prefix();
        self.log(s);
    }
    
    pub fn prefixed_logln(&self, s: &str) {
        self.write_prefix();
        self.logln(s);
    }

    pub fn logln(&self, s: &str) {
        self.log_fmt(format_args!("{s}\n"));
    }

    pub fn push_indent(&mut self) {
        self.indents.push(true);
    }

    pub fn pop_indent(&mut self) {
        self.indents.pop();
    }

    pub fn set_last_at_indent(&mut self) {
        if let Some(val) = self.indents.last_mut() {
            *val = false;
        }
    }

    pub fn set_last_at_indent_if(&mut self, cond: bool) {
        if cond {
            if let Some(val) = self.indents.last_mut() {
                *val = false;
            }
        }
    }

    pub fn log_indented<F>(&mut self, name: &str, f: F) where
        F: Fn(&mut Self)
    {
        self.prefixed_logln(name);
        self.push_indent();
        f(self);
        self.pop_indent();
    }
}