use eframe::egui::Vec2;
use std::*;

const TEMPLATE_STROKES: [(char, &[u8]); 26] = [
    ('A', b"\x00\x26\x40"),
    ('B', b"\x06\x00\x05\x16\x36\x45\x44\x33\x42\x41\x30\x10"),
    ('C', b"\x46\x16\x05\x01\x10\x40"),
    ('D', b"\x06\x00\x05\x16\x26\x44\x42\x20\x10"),
    ('E', b"\x46\x16\x05\x04\x13\x02\x01\x10\x40"),
    ('F', b"\x46\x06\x00"),
    ('G', b"\x46\x16\x05\x01\x10\x30\x41\x42\x23\x43"),
    ('H', b"\x06\x00\x01\x12\x32\x41\x40"),
    ('I', b"\x06\x00"),
    ('J', b"\x46\x41\x30\x10\x01\x03"),
    ('K', b"\x46\x12\x02\x04\x14\x40"),
    ('L', b"\x06\x00\x40"),
    ('M', b"\x00\x06\x23\x46\x40"),
    ('N', b"\x00\x06\x40\x46"),
    ('O', b"\x26\x16\x05\x01\x10\x30\x41\x45\x36\x26"),
    ('P', b"\x06\x00\x05\x16\x36\x45\x34\x14"),
    ('Q', b"\x36\x16\x05\x01\x10\x30\x41\x45\x36\x26\x46"),
    ('R', b"\x06\x00\x05\x16\x36\x45\x44\x33\x23\x40"),
    ('S', b"\x46\x16\x05\x04\x13\x33\x42\x41\x30\x00"),
    ('T', b"\x06\x46\x40"),
    ('U', b"\x06\x01\x10\x30\x41\x46"),
    ('V', b"\x06\x10\x26\x46"),
    ('W', b"\x06\x00\x23\x40\x46"),
    ('X', b"\x06\x32\x42\x44\x34\x00"),
    ('Y', b"\x06\x13\x33\x46\x30\x10\x12\x42"),
    ('Z', b"\x06\x46\x00\x40"),
];

pub struct Engine {
    templates: Vec<(char, Vec<Vec2>)>,
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

fn tangents_from_stroke(stroke: &[Vec2], n: usize) -> Vec<Vec2> {
    let len: f32 = (1..stroke.len()).map(|i| (stroke[i] - stroke[i - 1]).length()).sum();
    if len <= 0.0 {
        return Vec::new();
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
    dst
}

fn tangents_distance(ta: &[Vec2], tb: &[Vec2], penalty: f32) -> f32 {
    let mut dp = vec![-f32::INFINITY; tb.len() + 1];
    let mut s0: f32 = 0.0;
    for j in 0..ta.len() {
        for i in 0..tb.len() {
            let s1 = f32::max(s0, f32::max(dp[i + 1], dp[i]) - penalty) + (Vec2::dot(tb[i], ta[j]) - 1.0);
            s0 = mem::replace(&mut dp[i + 1], s1);
        }
        s0 = -f32::INFINITY;
    }
    *dp.last().unwrap() / (2.0 * cmp::max(ta.len(), tb.len()) as f32)
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            templates: TEMPLATE_STROKES
                .iter()
                .map(|(c, s)| (*c, tangents_from_stroke(&stroke_from_bytes(s), 64)))
                .collect(),
        }
    }

    pub fn classify_2d(&self, stroke: &Vec<Vec2>) -> Option<char> {
        let input = tangents_from_stroke(stroke, self.templates[0].1.len());
        if input.is_empty() {
            return None;
        }

        let mut best_letter = None;
        let mut best_score = -0.25;
        for (letter, template) in self.templates.iter() {
            let score = tangents_distance(&input, &template, 0.25);
            if score > best_score {
                best_score = score;
                best_letter = Some(*letter);
            }
            eprintln!("{}: {}", letter, score);
        }

        best_letter
    }
}
