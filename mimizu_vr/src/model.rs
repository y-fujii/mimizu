use std::*;

type Vector2 = nalgebra::Vector2<f32>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CharClass {
    Alphabet,
    Hiragana,
}

pub struct Model {
    pub recognizer: mimizu::GraffitiRecognizer,
    pub current_strokes: [Vec<Vector2>; 2],
    pub text: Vec<char>,
    pub cursor: usize,
    pub is_active: bool,
    pub use_chatbox: bool,
    pub char_class: CharClass,
}

impl Model {
    pub fn new() -> Self {
        Model {
            recognizer: mimizu::GraffitiRecognizer::new(0.02),
            current_strokes: [Vec::new(), Vec::new()],
            text: Vec::new(),
            cursor: 0,
            is_active: false,
            use_chatbox: true,
            char_class: CharClass::Alphabet,
        }
    }

    pub fn feed_stroke(&mut self, stroke: &[Vector2], _mode: mimizu::GraffitiMode) {
        dbg!(_mode);
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

    pub fn text_l(&self) -> String {
        self.translate(self.text[..self.cursor].iter().collect())
    }

    pub fn text_r(&self) -> String {
        self.translate(self.text[self.cursor..].iter().collect())
    }

    fn translate(&self, s: String) -> String {
        use wana_kana::ConvertJapanese;
        match self.char_class {
            CharClass::Alphabet => s,
            CharClass::Hiragana => s.to_hiragana(),
        }
    }
}
