use crate::recognizer::*;
use crate::{templates, Vector2};
use std::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GraffitiMode {
    Alphabet,
    Number,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GraffitiModifier {
    None,
    Symbol,
    Caps,
}

pub struct GraffitiRecognizer {
    alphabets: Recognizer,
    numbers: Recognizer,
    symbols: Recognizer,
    tap_tolerance: f32,
    mode: GraffitiMode,
    modifier: GraffitiModifier,
}

pub(crate) fn stroke_from_bytes(bytes: &[u8]) -> Vec<Vector2> {
    let mut dst = Vec::new();
    for byte in bytes.iter() {
        let x = (byte >> 4) as f32;
        let y = (byte & 0xf) as f32;
        dst.push(Vector2::new(x, y));
    }
    dst
}

impl GraffitiRecognizer {
    pub fn new(tap_tolerance: f32) -> Self {
        let n = 64;
        let mut alphabets = Recognizer::new(n);
        for (_, t) in templates::ALPHABETS.iter() {
            alphabets.add_template(&stroke_from_bytes(t));
        }
        let mut numbers = Recognizer::new(n);
        for (_, t) in templates::NUMBERS.iter() {
            numbers.add_template(&stroke_from_bytes(t));
        }
        let mut symbols = Recognizer::new(n);
        for (_, t) in templates::SYMBOLS.iter() {
            symbols.add_template(&stroke_from_bytes(t));
        }

        Self {
            alphabets,
            numbers,
            symbols,
            tap_tolerance,
            mode: GraffitiMode::Alphabet,
            modifier: GraffitiModifier::None,
        }
    }

    pub fn recognize(&mut self, stroke: &[Vector2]) -> Option<char> {
        if stroke.is_empty() {
            return None;
        }

        if stroke_len(stroke) <= self.tap_tolerance {
            return match self.modifier {
                GraffitiModifier::Symbol => {
                    self.modifier = GraffitiModifier::None;
                    Some('.')
                }
                _ => {
                    self.modifier = GraffitiModifier::Symbol;
                    None
                }
            };
        }

        let (recognizer, template): (_, &[_]) = match self.modifier {
            GraffitiModifier::Symbol => (&self.symbols, &templates::SYMBOLS),
            _ => match self.mode {
                GraffitiMode::Alphabet => (&self.alphabets, &templates::ALPHABETS),
                GraffitiMode::Number => (&self.numbers, &templates::NUMBERS),
            },
        };
        let Some(i) = recognizer.recognize(stroke) else {
            return None;
        };

        match template[i].0 {
            'N' => {
                self.mode = GraffitiMode::Number;
                self.modifier = GraffitiModifier::None;
                None
            }
            'A' => {
                self.mode = GraffitiMode::Alphabet;
                self.modifier = GraffitiModifier::None;
                None
            }
            'C' => {
                self.modifier = GraffitiModifier::Caps;
                None
            }
            c => {
                let c = match self.modifier {
                    GraffitiModifier::Caps => c.to_ascii_uppercase(),
                    _ => c,
                };
                self.modifier = GraffitiModifier::None;
                Some(c)
            }
        }
    }

    pub fn mode(&self) -> GraffitiMode {
        self.mode
    }

    pub fn modifier(&self) -> GraffitiModifier {
        self.modifier
    }
}
