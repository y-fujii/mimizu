use std::*;

type Vector2 = nalgebra::Vector2<f32>;
type Vector3 = nalgebra::Vector3<f32>;
type Vector4 = nalgebra::Vector4<f32>;
type Matrix3 = nalgebra::Matrix3<f32>;
type Matrix2x3 = nalgebra::Matrix2x3<f32>;
type Matrix3x4 = nalgebra::Matrix3x4<f32>;

pub struct StrokeProjector {
    stroke: Vec<Vector3>,
    hand: Matrix3x4,
    head: Matrix3x4,
}

pub(crate) fn project_to_plane(stroke3: &[Vector3], up: Vector3, front: Vector3) -> Vec<Vector2> {
    let stabilizer = 8.0;

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
        (cov.trace() / (6.0 * stabilizer)) * (ex * ex.transpose() + ey * ey.transpose())
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

    let mut stroke2 = Vec::new();
    for v3 in stroke3.iter() {
        stroke2.push(mp * v3);
    }

    stroke2
}

impl StrokeProjector {
    pub fn new() -> Self {
        StrokeProjector {
            stroke: Vec::new(),
            hand: num_traits::Zero::zero(),
            head: num_traits::Zero::zero(),
        }
    }

    pub fn clear(&mut self) {
        self.stroke.clear();
        self.hand = num_traits::Zero::zero();
        self.head = num_traits::Zero::zero();
    }

    pub fn feed(&mut self, hand: &Matrix3x4, head: &Matrix3x4) {
        self.stroke.push(hand * Vector4::new(0.0, 0.0, 0.0, 1.0));
        self.hand += hand;
        self.head += head;
    }

    pub fn stroke(&self) -> Vec<Vector2> {
        let ey = self.head * Vector4::y() + self.stroke.len() as f32 * Vector3::y();
        let ez = self.head * Vector4::new(0.0, 0.0, 1.0, 1.0)
            + self.hand * Vector4::new(0.0, 0.0, 1.0, -1.0);
        project_to_plane(&self.stroke, ey, ez)
    }
}
