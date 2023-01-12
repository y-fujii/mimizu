use std::*;

type Vector2 = nalgebra::Vector2<f32>;
type Vector3 = nalgebra::Vector3<f32>;
type Matrix3 = nalgebra::Matrix3<f32>;
type Matrix2x3 = nalgebra::Matrix2x3<f32>;

pub fn project_to_plane(stroke3: &[Vector3], up: Vector3, front: Vector3) -> Vec<Vector2> {
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
        cov.trace() * (ex * ex.transpose() + ey * ey.transpose())
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
