use crate::recognizer::*;
use crate::templates;
use std::*;

pub struct GraffitiRecognizer {
    recognizer_alphabets: Recognizer,
    recognizer_numbers: Recognizer,
    recognizer_symbols: Recognizer,
    pub tap_tolerance: f32,
    pub mode_number: bool,
    pub next_symbol: bool,
    pub next_caps: bool,
}

pub fn stroke_from_bytes(bytes: &[u8]) -> Vec<Vec2> {
    let mut dst = Vec::new();
    for byte in bytes.iter() {
        let x = (byte >> 4) as f32;
        let y = (byte & 0xf) as f32;
        dst.push([x, y]);
    }
    dst
}

impl GraffitiRecognizer {
    pub fn new(tap_tolerance: f32) -> Self {
        let n = 64;
        let mut recognizer_alphabets = Recognizer::new(n);
        for (_, t) in templates::ALPHABETS.iter() {
            recognizer_alphabets.add_template(&stroke_from_bytes(t));
        }
        let mut recognizer_numbers = Recognizer::new(n);
        for (_, t) in templates::NUMBERS.iter() {
            recognizer_numbers.add_template(&stroke_from_bytes(t));
        }
        let mut recognizer_symbols = Recognizer::new(n);
        for (_, t) in templates::SYMBOLS.iter() {
            recognizer_symbols.add_template(&stroke_from_bytes(t));
        }

        Self {
            recognizer_alphabets,
            recognizer_numbers,
            recognizer_symbols,
            tap_tolerance,
            mode_number: false,
            next_symbol: false,
            next_caps: false,
        }
    }

    pub fn recognize(&mut self, stroke: &[Vec2]) -> Option<char> {
        if stroke.is_empty() {
            return None;
        }
        if stroke_len(stroke) <= self.tap_tolerance {
            return if self.next_symbol {
                self.next_symbol = false;
                Some('.')
            } else {
                self.next_symbol = true;
                None
            };
        }

        let (template, recognizer) = if self.next_symbol {
            (&templates::SYMBOLS[..], &self.recognizer_symbols)
        } else {
            if self.mode_number {
                (&templates::NUMBERS[..], &self.recognizer_numbers)
            } else {
                (&templates::ALPHABETS[..], &self.recognizer_alphabets)
            }
        };

        let letter = recognizer.recognize(stroke).map(|i| template[i].0);

        match letter {
            Some('N') => {
                self.mode_number = true;
                self.next_symbol = false;
                self.next_caps = false;
                None
            }
            Some('A') => {
                self.mode_number = false;
                self.next_symbol = false;
                self.next_caps = false;
                None
            }
            Some('C') => {
                self.next_symbol = false;
                self.next_caps = !self.next_caps;
                None
            }
            Some(c) => {
                let c = if self.next_caps {
                    c.to_ascii_uppercase()
                } else {
                    c
                };
                self.next_symbol = false;
                self.next_caps = false;
                Some(c)
            }
            None => None,
        }
    }
}
