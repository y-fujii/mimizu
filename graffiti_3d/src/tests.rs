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
