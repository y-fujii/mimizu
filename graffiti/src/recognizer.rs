use std::*;

pub(crate) type Vec2 = [f32; 2];

pub struct Recognizer {
    n_samples: usize,
    templates: Vec<Vec<Vec2>>,
}

pub(crate) fn sub(x: Vec2, y: Vec2) -> Vec2 {
    [x[0] - y[0], x[1] - y[1]]
}

pub(crate) fn dot(x: Vec2, y: Vec2) -> f32 {
    x[0] * y[0] + x[1] * y[1]
}

pub(crate) fn norm(x: Vec2) -> f32 {
    f32::sqrt(dot(x, x))
}

pub(crate) fn normalize(x: Vec2) -> Vec2 {
    let n = norm(x);
    [x[0] / n, x[1] / n]
}

pub(crate) fn stroke_len(stroke: &[Vec2]) -> f32 {
    (1..stroke.len())
        .map(|i| norm(sub(stroke[i], stroke[i - 1])))
        .sum()
}

pub(crate) fn tangents_from_stroke(stroke: &[Vec2], n: usize) -> Vec<Vec2> {
    let len = stroke_len(stroke);
    if len <= 0.0 {
        return Vec::new();
    }

    let mut dst = Vec::new();
    let mut pos = 0.0;
    let mut i = 0;
    let mut j = 0;
    while j < n {
        if n as f32 * pos <= (j as f32 + 0.5) * len {
            pos += norm(sub(stroke[i + 1], stroke[i]));
            i += 1;
        } else {
            dst.push(normalize(sub(stroke[i], stroke[i - 1])));
            j += 1;
        }
    }
    dst
}

// f(a, b) == f(b, a), f(a, a) == 1, -1 <= f(a, b) <= 1.
pub(crate) fn tangents_similarity(ta: &[Vec2], tb: &[Vec2], penalty: f32) -> f32 {
    let mut dps = vec![(0.0, -f32::INFINITY); tb.len() + 1];
    let mut dp0 = (0.0, 0.0);
    for i in 0..ta.len() {
        for j in 0..tb.len() {
            let s = dot(tb[j], ta[i]);
            let v0 = dp0.1 + 0.5 * (dp0.0 + s);
            let v1 = dps[j + 1].1 + 0.25 * (dps[j + 1].0 + s) - penalty;
            let v2 = dps[j + 0].1 + 0.25 * (dps[j + 0].0 + s) - penalty;
            dp0 = mem::replace(&mut dps[j + 1], (s, v0.max(v1).max(v2)));
        }
        dp0 = (0.0, -f32::INFINITY);
    }
    let v = dps.last().unwrap().1 + 0.5 * dot(*tb.last().unwrap(), *ta.last().unwrap());
    v / cmp::max(ta.len(), tb.len()) as f32
}

impl Recognizer {
    pub fn new(n: usize) -> Self {
        Self {
            n_samples: n,
            templates: Vec::new(),
        }
    }

    pub fn add_template(&mut self, stroke: &[Vec2]) {
        self.templates
            .push(tangents_from_stroke(stroke, self.n_samples));
    }

    pub fn recognize(&self, stroke: &[Vec2]) -> Option<usize> {
        let input = tangents_from_stroke(stroke, self.n_samples);
        if input.is_empty() {
            return None;
        }

        let mut best_idx = None;
        let mut best_sim = 0.0;
        for (i, template) in self.templates.iter().enumerate() {
            let sim = tangents_similarity(&input, &template, 0.25);
            if sim > best_sim {
                best_sim = sim;
                best_idx = Some(i);
            }
        }

        best_idx
    }

    pub fn recognize_all(&self, stroke: &[Vec2]) -> Vec<f32> {
        let input = tangents_from_stroke(stroke, self.n_samples);
        if input.is_empty() {
            return vec![-1.0; self.templates.len()];
        }

        self.templates
            .iter()
            .map(|t| tangents_similarity(&input, &t, 0.25))
            .collect()
    }
}
