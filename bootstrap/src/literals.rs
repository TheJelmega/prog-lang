use core::ops::Index;
use std::fmt::Display;


#[derive(PartialEq, Debug)]
pub enum Literal {
    Decimal {
        int_digits: Vec<u8>,
        frac_digits: Vec<u8>,
        exp_sign: bool,
        exp_digits: Vec<u8>,
    },
    /// Binary literal, bytes contain sets of 8 binary digits
    Binary{ bytes: Vec<u8> },
    /// Octal literal, digits contain pairs of digits, upper: 6..=4, lower: 2..=0 (i.e. as nibbles)
    Octal{ digits: Vec<u8> },
    /// Integer hexadecimal nibbles
    HexInt{ nibbles: Vec<u8> },
    /// Floating-point hexadecimal nibbles
    HexFp {
        initial_digit: bool,
        mantissa: Vec<u8>,
        exp_sign: bool,
        exponent: Vec<u8>,
    },
    Char(char),
    String(String),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Decimal { int_digits, frac_digits, exp_sign, exp_digits } => {
                print_digits(f, int_digits, false)?;
                if !frac_digits.is_empty() {
                    write!(f, ".")?;
                    print_digits(f, frac_digits, true)?;

                    if !exp_digits.is_empty() {
                        write!(f, "e{}", if *exp_sign { '+' } else { '-' })?;
                        print_digits(f, exp_digits, false)?;
                    }
                }

                Ok(())
            },
            Literal::Binary { bytes } => {
                write!(f, "0b")?;
                for (idx, byte) in bytes.iter().enumerate() {
                    if idx == 0 {
                        write!(f, "{byte:b}")?;
                    } else {
                        write!(f, "{byte:08b}")?;
                    }
                }
                Ok(())
            },
            Literal::Octal { digits } => {
                write!(f, "0o")?;
                print_digits(f, digits, false)
            },
            Literal::HexInt { nibbles } => {
                write!(f, "0x")?;
                print_digits(f, nibbles, false)
            },
            Literal::HexFp { initial_digit, mantissa, exp_sign, exponent } => {
                write!(f, "0x{}.", if *initial_digit { '1' } else { '0' })?;
                print_digits(f, mantissa, true)?;
                write!(f, "p{}", if *exp_sign { '+' } else { '-' })?;
                print_digits(f, exponent, false)
            },
            Literal::Char(ch) => write!(f, "'{ch}'"),
            Literal::String(s) => write!(f, "\"{s}\""),
        }
    }
}

fn print_digits(f: &mut std::fmt::Formatter<'_>, digits: &Vec<u8>, show_preceding_zeros: bool) -> std::fmt::Result {
    for (idx, pair) in digits.iter().enumerate() {
        let first = pair >> 4;
        let second = pair & 0xF;

        if idx != 0 || first != 0 || show_preceding_zeros {
            write!(f, "{first:X}")?;
        }
        write!(f, "{second:X}")?;
    }

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LiteralId(u32);

impl Display for LiteralId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct LiteralTable {
    literals: Vec<Literal>,
}

impl LiteralTable {
    pub fn new() -> Self {
        Self {
            literals: Vec::new(),
        }
    }

    pub fn add(&mut self, lit: Literal) -> LiteralId {
        // Naive implementation
        let it = self.literals.iter().enumerate().find(|(_, val)| **val == lit);
        match it {
            Some((idx, _)) => LiteralId(idx as u32),
            None => {
                let idx = self.literals.len() as u32;
                self.literals.push(lit);
                LiteralId(idx)
            }
        }
    }

    pub fn get(&self, id: LiteralId) -> &Literal {
        &self.literals[id.0 as usize]
    }
}

impl Index<LiteralId> for LiteralTable {
    type Output = Literal;

    fn index(&self, index: LiteralId) -> &Self::Output {
        self.get(index)
    }
}