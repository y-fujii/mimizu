use crate::*;

#[test]
fn test_tangents_similarity() {
    let ex = Vector2::new(1.0, 0.0);
    let ey = Vector2::new(0.0, 1.0);
    assert_eq!(tangents_similarity(&[ex], &[ex], 1.0), 1.0);
    assert_eq!(tangents_similarity(&[ex], &[ey], 1.0), 0.0);
    assert_eq!(tangents_similarity(&[ex, ey], &[ex, ex], 1.0), 1.0 / 2.0);
    assert_eq!(tangents_similarity(&[ex, ey], &[ey, ey], 1.0), 1.0 / 2.0);
    assert_eq!(tangents_similarity(&[ex, ey], &[ex, ex], 0.0), 5.0 / 8.0);
    assert_eq!(tangents_similarity(&[ex, ey], &[ey, ey], 0.0), 5.0 / 8.0);
    assert_eq!(tangents_similarity(&[ex, ex, ey], &[ex, ey, ey], 0.0), 1.0);
    assert_eq!(
        tangents_similarity(&[ex, ex, ex, ey], &[ex, ey, ey, ey], 0.0),
        1.0
    );
    assert_eq!(
        tangents_similarity(&[ex, ex, ey], &[ex, ey, ey], 1.0),
        2.0 / 3.0
    );
    assert_eq!(
        tangents_similarity(&[ex, ex, ey], &[ex, ey, ey], 0.25),
        2.5 / 3.0
    );
}

#[test]
fn test_tangents_from_stroke() {
    let e00 = Vector2::new(0.0, 0.0);
    let e10 = Vector2::new(1.0, 0.0);
    let e01 = Vector2::new(0.0, 1.0);
    let e11 = Vector2::new(1.0, 1.0);
    assert_eq!(tangents_from_stroke(&[e00, e10], 1), [e10; 1]);
    assert_eq!(tangents_from_stroke(&[e00, e10], 2), [e10; 2]);
    assert_eq!(tangents_from_stroke(&[e00, e10], 4), [e10; 4]);
    assert_eq!(tangents_from_stroke(&[-e10, e00, e10], 1), [e10; 1]);
    assert_eq!(tangents_from_stroke(&[-e10, e00, e10], 2), [e10; 2]);
    assert_eq!(tangents_from_stroke(&[-e10, e00, e10], 4), [e10; 4]);
    assert_eq!(
        tangents_from_stroke(&[e00, e10, 2.0 * e10, 3.0 * e10], 1),
        [e10; 1]
    );
    assert_eq!(
        tangents_from_stroke(&[e00, e10, 2.0 * e10, 3.0 * e10], 2),
        [e10; 2],
    );
    assert_eq!(
        tangents_from_stroke(&[e00, e10, 2.0 * e10, 3.0 * e10], 4),
        [e10; 4],
    );
    assert_eq!(tangents_from_stroke(&[e00, e10, e11], 1), [0.5 * e11]);
    assert_eq!(tangents_from_stroke(&[e00, e10, e11], 2), [e10, e01]);
    assert_eq!(
        tangents_from_stroke(&[e00, e10, e11], 4),
        [e10, e10, e01, e01]
    );
}
