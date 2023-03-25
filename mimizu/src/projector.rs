// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::{Matrix2x3, Matrix3, Matrix3x4, Vector2, Vector3, Vector4};
use std::*;

pub struct StrokeProjector {
    stroke: Vec<Vector3>,
    ey_sum: Vector3,
    ez_sum: Vector3,
}

pub(crate) fn project_to_plane(
    stroke3: &[Vector3],
    up: Vector3,
    front: Vector3,
    sf: f32,
) -> Vec<Vector2> {
    if stroke3.is_empty() {
        return Vec::new();
    }

    let mean = stroke3.iter().sum::<Vector3>() / stroke3.len() as f32;
    let cov = stroke3
        .iter()
        .map(|v| (v - mean) * (v - mean).transpose())
        .sum::<Matrix3>();

    // stabilize the plane when the stroke is almost 1-d.
    let cov = cov + {
        let ez = front;
        let ex = up.cross(&ez).normalize();
        let ey = ez.cross(&ex).normalize();
        (sf / 6.0) * cov.trace() * (ex * ex.transpose() + ey * ey.transpose())
    };

    // XXX: use inverse power method.
    let ez: Vector3 = {
        let eigens = cov.symmetric_eigen();
        let mut min_v = f32::INFINITY;
        let mut min_i = usize::MAX;
        for (i, &v) in eigens.eigenvalues.iter().enumerate() {
            if v <= min_v {
                min_v = v;
                min_i = i;
            }
        }
        eigens.eigenvectors.column(min_i).into()
    };

    let ez = if ez.dot(&front) < 0.0 { -ez } else { ez };
    let ex = up.cross(&ez).normalize();
    let ey = ez.cross(&ex).normalize();
    let mp = Matrix2x3::from_rows(&[ex.transpose(), ey.transpose()]);
    stroke3.iter().map(|v3| mp * v3).collect()
}

impl StrokeProjector {
    pub fn new() -> Self {
        StrokeProjector {
            stroke: Vec::new(),
            ey_sum: nalgebra::zero(),
            ez_sum: nalgebra::zero(),
        }
    }

    pub fn clear(&mut self) {
        self.stroke.clear();
        self.ey_sum = nalgebra::zero();
        self.ez_sum = nalgebra::zero();
    }

    pub fn feed(&mut self, hand: &Matrix3x4, head: &Matrix3x4) {
        let ey0 = (head * Vector4::y()).normalize();
        let ey1 = Vector3::y();
        let ez0 = ((head - hand) * Vector4::w()).normalize();
        let ez1 = (head * Vector4::z()).normalize();
        let ez2 = (hand * Vector4::z()).normalize();
        self.ey_sum += ey0 + ey1;
        self.ez_sum += ez0 + ez1 + ez2;
        self.stroke.push(hand * Vector4::w());
    }

    pub fn stroke(&self) -> Vec<Vector2> {
        project_to_plane(&self.stroke, self.ey_sum, self.ez_sum, 1.0 / 8.0)
    }
}
