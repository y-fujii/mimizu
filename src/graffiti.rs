use crate::template;
use eframe::egui::Vec2;
use std::*;

pub struct Engine {
    alphabets: Vec<(char, Vec<Vec2>)>,
    numbers: Vec<(char, Vec<Vec2>)>,
    symbols: Vec<(char, Vec<Vec2>)>,
    pub tap_tolerance: f32,
    pub mode_number: bool,
    pub next_symbol: bool,
    pub next_caps: bool,
}

fn stroke_from_bytes(bytes: &[u8]) -> Vec<Vec2> {
    let mut dst = Vec::new();
    for byte in bytes.iter() {
        let x = (byte >> 4) as f32;
        let y = (byte & 0xf) as f32;
        dst.push(Vec2::new(x, -y));
    }
    dst
}

fn tangents_from_stroke(stroke: &[Vec2], n: usize) -> (Vec<Vec2>, f32) {
    let len: f32 = (1..stroke.len())
        .map(|i| (stroke[i] - stroke[i - 1]).length())
        .sum();
    if len <= 0.0 {
        return (Vec::new(), 0.0);
    }

    let mut dst = Vec::new();
    let mut pos = 0.0;
    let mut i = 0;
    let mut j = 0;
    while j < n {
        if n as f32 * pos <= (j as f32 + 0.5) * len {
            pos += (stroke[i + 1] - stroke[i]).length();
            i += 1;
        } else {
            dst.push((stroke[i] - stroke[i - 1]).normalized());
            j += 1;
        }
    }
    (dst, len)
}

// f(a, b) == f(b, a), f(a, a) == 1, -1 <= f(a, b) <= 1.
fn tangents_similarity(ta: &[Vec2], tb: &[Vec2], penalty: f32) -> f32 {
    let mut dps = vec![(0.0, -f32::INFINITY); tb.len() + 1];
    let mut dp0 = (0.5 * Vec2::dot(tb[0], ta[0]), 0.0);
    for i in 0..ta.len() {
        for j in 0..tb.len() {
            let s = Vec2::dot(tb[j], ta[i]);
            let v0 = dp0.1 + 0.5 * (dp0.0 + s);
            let v1 = dps[j + 1].1 + 0.25 * (dps[j + 1].0 + s) - penalty;
            let v2 = dps[j + 0].1 + 0.25 * (dps[j + 0].0 + s) - penalty;
            dp0 = mem::replace(&mut dps[j + 1], (s, v0.max(v1).max(v2)));
        }
        dp0 = (0.0, -f32::INFINITY);
    }
    let v = dps.last().unwrap().1 + 0.5 * Vec2::dot(*tb.last().unwrap(), *ta.last().unwrap());
    v / cmp::max(ta.len(), tb.len()) as f32
}

impl Engine {
    pub fn new(tap_tolerance: f32) -> Self {
        let n = 64;
        Engine {
            alphabets: template::ALPHABETS
                .iter()
                .map(|(c, s)| (*c, tangents_from_stroke(&stroke_from_bytes(s), n).0))
                .collect(),
            numbers: template::NUMBERS
                .iter()
                .map(|(c, s)| (*c, tangents_from_stroke(&stroke_from_bytes(s), n).0))
                .collect(),
            symbols: template::SYMBOLS
                .iter()
                .map(|(c, s)| (*c, tangents_from_stroke(&stroke_from_bytes(s), n).0))
                .collect(),
            tap_tolerance: tap_tolerance,
            mode_number: false,
            next_symbol: false,
            next_caps: false,
        }
    }

    pub fn classify_2d(&mut self, stroke: &Vec<Vec2>) -> Option<char> {
        if stroke.is_empty() {
            return None;
        }
        let (input, len) = tangents_from_stroke(stroke, self.alphabets[0].1.len());
        if len <= self.tap_tolerance {
            if self.next_symbol {
                self.next_symbol = false;
                return Some('.');
            } else {
                self.next_symbol = true;
                return None;
            }
        }

        let templates = if self.next_symbol {
            &self.symbols
        } else {
            if self.mode_number {
                &self.numbers
            } else {
                &self.alphabets
            }
        };

        let mut best_letter = None;
        let mut best_sim = 0.0;
        for (letter, template) in templates.iter() {
            let sim = tangents_similarity(&input, &template, 0.25);
            if sim > best_sim {
                best_sim = sim;
                best_letter = Some(*letter);
            }
        }

        match best_letter {
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
