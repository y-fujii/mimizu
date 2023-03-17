use std::*;
//use wana_kana::ConvertJapanese;

type Vector2 = nalgebra::Vector2<f32>;

pub struct Model {
    pub recognizer: graffiti_3d::GraffitiRecognizer,
    pub current_strokes: [Vec<Vector2>; 2],
    pub text: Vec<char>,
    pub cursor: usize,
}

impl Model {
    pub fn new() -> Self {
        Model {
            recognizer: graffiti_3d::GraffitiRecognizer::new(0.02),
            current_strokes: [Vec::new(), Vec::new()],
            text: Vec::new(),
            cursor: 0,
        }
    }

    pub fn feed_stroke(&mut self, stroke: &[Vector2]) {
        let Some(c) = self.recognizer.recognize(&stroke) else {
            return;
        };
        match c {
            '\x08' => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.text.remove(self.cursor);
                }
            }
            '←' => {
                self.cursor = cmp::max(self.cursor, 1) - 1;
            }
            '→' => {
                self.cursor = cmp::min(self.cursor + 1, self.text.len());
            }
            '\n' => {
                self.text.clear();
                self.cursor = 0;
            }
            c => {
                self.text.insert(self.cursor, c);
                self.cursor += 1;
            }
        }
    }
}
