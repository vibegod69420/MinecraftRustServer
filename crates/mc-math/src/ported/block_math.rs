use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    // JAVA: net.minecraft.core.Direction#getApproximateNearest
    pub fn get_approximate_nearest(x: f32, y: f32, z: f32) -> Direction {
        let mut best = Direction::North;
        let mut best_val = f32::MIN;
        for &dir in &[
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            let (dx, dy, dz) = dir.normal();
            let dot = x * dx + y * dy + z * dz;
            if dot > best_val {
                best_val = dot;
                best = dir;
            }
        }
        best
    }

    fn normal(self) -> (f32, f32, f32) {
        match self {
            Direction::Down => (0.0, -1.0, 0.0),
            Direction::Up => (0.0, 1.0, 0.0),
            Direction::North => (0.0, 0.0, -1.0),
            Direction::South => (0.0, 0.0, 1.0),
            Direction::West => (-1.0, 0.0, 0.0),
            Direction::East => (1.0, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Column-major 4x4 matrix, matching JOML layout: m[col * 4 + row].
#[derive(Clone, Copy, Debug)]
pub struct Matrix4f {
    pub m: [f32; 16],
}

impl Matrix4f {
    pub fn identity() -> Self {
        let mut m = [0.0f32; 16];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[15] = 1.0;
        Self { m }
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut mat = Self::identity();
        mat.m[12] = x;
        mat.m[13] = y;
        mat.m[14] = z;
        mat
    }

    pub fn mul(&mut self, other: &Matrix4f) -> &mut Self {
        let a = self.m;
        let b = other.m;
        let mut r = [0.0f32; 16];
        for col in 0..4 {
            for row in 0..4 {
                r[col * 4 + row] = a[row] * b[col * 4]
                    + a[4 + row] * b[col * 4 + 1]
                    + a[8 + row] * b[col * 4 + 2]
                    + a[12 + row] * b[col * 4 + 3];
            }
        }
        self.m = r;
        self
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let t = Matrix4f::translation(x, y, z);
        // JOML Matrix4f.translate applies translation: this = this * T
        let mut tmp = *self;
        tmp.mul(&t);
        *self = tmp;
        self
    }

    pub fn transform_direction(&self, v: Vector3f) -> Vector3f {
        let m = &self.m;
        Vector3f::new(
            m[0] * v.x + m[4] * v.y + m[8] * v.z,
            m[1] * v.x + m[5] * v.y + m[9] * v.z,
            m[2] * v.x + m[6] * v.y + m[10] * v.z,
        )
    }

    pub fn rotation_x(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let mut m = Self::identity();
        m.m[5] = c;
        m.m[6] = s;
        m.m[9] = -s;
        m.m[10] = c;
        m
    }

    pub fn rotation_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let mut m = Self::identity();
        m.m[0] = c;
        m.m[2] = -s;
        m.m[8] = s;
        m.m[10] = c;
        m
    }

    pub fn inverse(&self) -> Self {
        let m = &self.m;
        let mut inv = [0.0f32; 16];

        inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
            + m[9] * m[7] * m[14]
            + m[13] * m[6] * m[11]
            - m[13] * m[7] * m[10];
        inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
            - m[8] * m[7] * m[14]
            - m[12] * m[6] * m[11]
            + m[12] * m[7] * m[10];
        inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
            + m[8] * m[7] * m[13]
            + m[12] * m[5] * m[11]
            - m[12] * m[7] * m[9];
        inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
            - m[8] * m[6] * m[13]
            - m[12] * m[5] * m[10]
            + m[12] * m[6] * m[9];

        inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
            - m[9] * m[3] * m[14]
            - m[13] * m[2] * m[11]
            + m[13] * m[3] * m[10];
        inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
            + m[8] * m[3] * m[14]
            + m[12] * m[2] * m[11]
            - m[12] * m[3] * m[10];
        inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
            - m[8] * m[3] * m[13]
            - m[12] * m[1] * m[11]
            + m[12] * m[3] * m[9];
        inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
            + m[8] * m[2] * m[13]
            + m[12] * m[1] * m[10]
            - m[12] * m[2] * m[9];

        inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
            + m[5] * m[3] * m[14]
            + m[13] * m[2] * m[7]
            - m[13] * m[3] * m[6];
        inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
            - m[4] * m[3] * m[14]
            - m[12] * m[2] * m[7]
            + m[12] * m[3] * m[6];
        inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
            + m[4] * m[3] * m[13]
            + m[12] * m[1] * m[7]
            - m[12] * m[3] * m[5];
        inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
            - m[4] * m[2] * m[13]
            - m[12] * m[1] * m[6]
            + m[12] * m[2] * m[5];

        inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
            - m[5] * m[3] * m[10]
            - m[9] * m[2] * m[7]
            + m[9] * m[3] * m[6];
        inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
            + m[4] * m[3] * m[10]
            + m[8] * m[2] * m[7]
            - m[8] * m[3] * m[6];
        inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
            - m[4] * m[3] * m[9]
            - m[8] * m[1] * m[7]
            + m[8] * m[3] * m[5];
        inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
            + m[4] * m[2] * m[9]
            + m[8] * m[1] * m[6]
            - m[8] * m[2] * m[5];

        let det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];
        if det == 0.0 {
            return Self::identity();
        }
        let inv_det = 1.0 / det;
        for v in &mut inv {
            *v *= inv_det;
        }
        Self { m: inv }
    }
}

pub struct MatrixUtil;

impl MatrixUtil {
    // JAVA: com.mojang.math.MatrixUtil#isIdentity
    pub fn is_identity(mat: &Matrix4f) -> bool {
        let id = Matrix4f::identity();
        mat.m == id.m
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Transformation {
    matrix: Matrix4f,
}

impl Transformation {
    // JAVA: com.mojang.math.Transformation#identity
    pub fn identity() -> Self {
        Self {
            matrix: Matrix4f::identity(),
        }
    }

    pub fn from_matrix(matrix: Matrix4f) -> Self {
        Self { matrix }
    }

    pub fn from_rotation_y(angle: f32) -> Self {
        Self {
            matrix: Matrix4f::rotation_y(angle),
        }
    }

    pub fn from_rotation_x(angle: f32) -> Self {
        Self {
            matrix: Matrix4f::rotation_x(angle),
        }
    }

    // JAVA: com.mojang.math.Transformation#getMatrix
    pub fn get_matrix(&self) -> &Matrix4f {
        &self.matrix
    }

    // JAVA: com.mojang.math.Transformation#inverse
    pub fn inverse(&self) -> Self {
        Self {
            matrix: self.matrix.inverse(),
        }
    }

    // JAVA: com.mojang.math.Transformation#compose
    pub fn compose(&self, other: &Transformation) -> Self {
        let mut m = self.matrix;
        m.mul(&other.matrix);
        Self { matrix: m }
    }
}

fn vanilla_uv_transform_local_to_global() -> &'static HashMap<Direction, Transformation> {
    static MAP: OnceLock<HashMap<Direction, Transformation>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(Direction::South, Transformation::identity());
        m.insert(
            Direction::East,
            Transformation::from_rotation_y(PI / 2.0),
        );
        m.insert(
            Direction::West,
            Transformation::from_rotation_y(-PI / 2.0),
        );
        m.insert(Direction::North, Transformation::from_rotation_y(PI));
        m.insert(
            Direction::Up,
            Transformation::from_rotation_x(-PI / 2.0),
        );
        m.insert(
            Direction::Down,
            Transformation::from_rotation_x(PI / 2.0),
        );
        m
    })
}

fn vanilla_uv_transform_global_to_local() -> &'static HashMap<Direction, Transformation> {
    static MAP: OnceLock<HashMap<Direction, Transformation>> = OnceLock::new();
    MAP.get_or_init(|| {
        vanilla_uv_transform_local_to_global()
            .iter()
            .map(|(k, v)| (*k, v.inverse()))
            .collect()
    })
}

// JAVA: net.minecraft.core.BlockMath#blockCenterToCorner
pub fn block_center_to_corner(t: &Transformation) -> Transformation {
    let mut m = Matrix4f::translation(0.5, 0.5, 0.5);
    m.mul(t.get_matrix());
    m.translate(-0.5, -0.5, -0.5);
    Transformation::from_matrix(m)
}

// JAVA: net.minecraft.core.BlockMath#blockCornerToCenter
pub fn block_corner_to_center(t: &Transformation) -> Transformation {
    let mut m = Matrix4f::translation(-0.5, -0.5, -0.5);
    m.mul(t.get_matrix());
    m.translate(0.5, 0.5, 0.5);
    Transformation::from_matrix(m)
}

// JAVA: net.minecraft.core.BlockMath#getFaceTransformation
pub fn get_face_transformation(t: &Transformation, dir: Direction) -> Transformation {
    if MatrixUtil::is_identity(t.get_matrix()) {
        return *t;
    }

    let local_to_global = vanilla_uv_transform_local_to_global();
    let global_to_local = vanilla_uv_transform_global_to_local();
    let face = local_to_global.get(&dir).expect("missing direction");
    let composed = t.compose(face);
    let v = composed
        .get_matrix()
        .transform_direction(Vector3f::new(0.0, 0.0, 1.0));
    let nearest = Direction::get_approximate_nearest(v.x, v.y, v.z);
    global_to_local
        .get(&nearest)
        .expect("missing direction")
        .compose(&composed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    fn mat_approx_eq(a: &Matrix4f, b: &Matrix4f) -> bool {
        a.m.iter().zip(b.m.iter()).all(|(x, y)| approx_eq(*x, *y))
    }

    #[test]
    fn identity_face_transformation_returns_same() {
        let t = Transformation::identity();
        let r = get_face_transformation(&t, Direction::South);
        assert!(MatrixUtil::is_identity(r.get_matrix()));
    }

    #[test]
    fn block_center_corner_roundtrip() {
        let t = Transformation::from_rotation_y(PI / 2.0);
        let corner = block_center_to_corner(&t);
        let center = block_corner_to_center(&corner);
        assert!(mat_approx_eq(center.get_matrix(), t.get_matrix()));
    }

    #[test]
    fn south_local_to_global_is_identity() {
        let m = vanilla_uv_transform_local_to_global()
            .get(&Direction::South)
            .unwrap();
        assert!(MatrixUtil::is_identity(m.get_matrix()));
    }

    #[test]
    fn get_approximate_nearest_basic() {
        assert_eq!(
            Direction::get_approximate_nearest(0.0, 1.0, 0.0),
            Direction::Up
        );
        assert_eq!(
            Direction::get_approximate_nearest(-0.7, 0.1, 0.0),
            Direction::West
        );
        assert_eq!(
            Direction::get_approximate_nearest(0.0, 0.0, -1.0),
            Direction::North
        );
    }

    #[test]
    fn inverse_of_rotation_y() {
        let t = Transformation::from_rotation_y(PI / 2.0);
        let inv = t.inverse();
        let composed = t.compose(&inv);
        assert!(mat_approx_eq(composed.get_matrix(), &Matrix4f::identity()));
    }
}
