// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::Vector2;
use std::*;

pub struct Recognizer {
    n_samples: usize,
    templates: Vec<Vec<Vector2>>,
}

pub(crate) fn stroke_len(stroke: &[Vector2]) -> f32 {
    (1..stroke.len())
        .map(|i| (stroke[i] - stroke[i - 1]).norm())
        .sum()
}

// results are not strictly normalized to improve robustness.
pub(crate) fn tangents_from_stroke(stroke: &[Vector2], n: usize) -> Vec<Vector2> {
    let len = stroke_len(stroke);
    if len <= 0.0 {
        return vec![nalgebra::zero(); n];
    }

    let mut dst = Vec::new();
    let mut p = stroke[0];
    let mut l = 0.0;
    let mut i = 0;
    let mut j = 1;
    while j < n {
        let dl = (stroke[i + 1] - stroke[i]).norm();
        let dt = (j as f32 / n as f32) * len - l;
        if dl <= dt {
            l += dl;
            i += 1;
        } else {
            let q = (1.0 - dt / dl) * stroke[i] + (dt / dl) * stroke[i + 1];
            dst.push((n as f32 / len) * (q - p));
            p = q;
            j += 1;
        }
    }
    if n > 0 {
        dst.push((n as f32 / len) * (stroke.last().unwrap() - p));
    }

    assert_eq!(dst.len(), n);
    dst
}

// f(a, b) == f(b, a), f(a, a) == 1, -1 <= f(a, b) <= 1.
pub(crate) fn tangents_similarity(ta: &[Vector2], tb: &[Vector2], penalty: f32) -> f32 {
    let mut dps = vec![(0.0, -f32::INFINITY); tb.len() + 1];
    let mut dp0 = (0.0, 0.0);
    for i in 0..ta.len() {
        for j in 0..tb.len() {
            let s = tb[j].dot(&ta[i]);
            let v0 = dp0.1 + 0.5 * (dp0.0 + s);
            let v1 = dps[j + 1].1 + 0.25 * (dps[j + 1].0 + s) - penalty;
            let v2 = dps[j + 0].1 + 0.25 * (dps[j + 0].0 + s) - penalty;
            dp0 = mem::replace(&mut dps[j + 1], (s, v0.max(v1).max(v2)));
        }
        dp0 = (0.0, -f32::INFINITY);
    }
    let v = dps.last().unwrap().1 + 0.5 * tb.last().unwrap().dot(ta.last().unwrap());
    v / cmp::max(ta.len(), tb.len()) as f32
}

impl Recognizer {
    pub fn new(n: usize) -> Self {
        Self {
            n_samples: n,
            templates: Vec::new(),
        }
    }

    pub fn add_template(&mut self, stroke: &[Vector2]) {
        self.templates
            .push(tangents_from_stroke(stroke, self.n_samples));
    }

    pub fn recognize(&self, stroke: &[Vector2]) -> Option<usize> {
        let input = tangents_from_stroke(stroke, self.n_samples);

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

    pub fn recognize_all(&self, stroke: &[Vector2]) -> Vec<f32> {
        let input = tangents_from_stroke(stroke, self.n_samples);

        self.templates
            .iter()
            .map(|t| tangents_similarity(&input, &t, 0.25))
            .collect()
    }
}
