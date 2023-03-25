// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
mod graffiti;
mod projector;
mod recognizer;
mod templates;
#[cfg(test)]
mod tests;

pub type Vector2 = nalgebra::Vector2<f32>;
pub type Vector3 = nalgebra::Vector3<f32>;
pub type Vector4 = nalgebra::Vector4<f32>;
pub type Matrix3 = nalgebra::Matrix3<f32>;
pub type Matrix2x3 = nalgebra::Matrix2x3<f32>;
pub type Matrix3x4 = nalgebra::Matrix3x4<f32>;

pub use crate::graffiti::*;
pub use crate::projector::*;
pub use crate::recognizer::*;
