use cgmath::{Matrix4, Vector4, vec4};

pub type Mat4 = Matrix4<i8>;
pub type Vec4 = Vector4<i8>;

pub const ZERO: Vec4 = vec4(0, 0, 0, 0);
pub const X: Vec4 = vec4(1, 0, 0, 0);
pub const Y: Vec4 = vec4(0, 1, 0, 0);
pub const Z: Vec4 = vec4(0, 0, 1, 0);
pub const W: Vec4 = vec4(0, 0, 0, 1);
