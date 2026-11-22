#[derive(Debug, Clone)]
pub struct Cursor3D {
    origin_x: i32,
    origin_y: i32,
    origin_z: i32,
    width: i32,
    height: i32,
    depth: i32,
    end: i32,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
}

pub const TYPE_INSIDE: i32 = 0;
pub const TYPE_FACE: i32 = 1;
pub const TYPE_EDGE: i32 = 2;
pub const TYPE_CORNER: i32 = 3;

impl Cursor3D {
    // JAVA: net.minecraft.core.Cursor3D#Cursor3D
    pub fn new(origin_x: i32, origin_y: i32, origin_z: i32, end_x: i32, end_y: i32, end_z: i32) -> Self {
        let width = end_x.wrapping_sub(origin_x).wrapping_add(1);
        let height = end_y.wrapping_sub(origin_y).wrapping_add(1);
        let depth = end_z.wrapping_sub(origin_z).wrapping_add(1);
        let end = width.wrapping_mul(height).wrapping_mul(depth);
        Self {
            origin_x,
            origin_y,
            origin_z,
            width,
            height,
            depth,
            end,
            index: 0,
            x: 0,
            y: 0,
            z: 0,
        }
    }

    // JAVA: net.minecraft.core.Cursor3D#advance
    pub fn advance(&mut self) -> bool {
        if self.index == self.end {
            return false;
        }
        self.x = self.index % self.width;
        let plane = self.index / self.width;
        self.y = plane % self.height;
        self.z = plane / self.height;
        self.index = self.index.wrapping_add(1);
        true
    }

    // JAVA: net.minecraft.core.Cursor3D#nextX
    pub fn next_x(&self) -> i32 {
        self.origin_x.wrapping_add(self.x)
    }

    // JAVA: net.minecraft.core.Cursor3D#nextY
    pub fn next_y(&self) -> i32 {
        self.origin_y.wrapping_add(self.y)
    }

    // JAVA: net.minecraft.core.Cursor3D#nextZ
    pub fn next_z(&self) -> i32 {
        self.origin_z.wrapping_add(self.z)
    }

    // JAVA: net.minecraft.core.Cursor3D#getNextType
    pub fn get_next_type(&self) -> i32 {
        let mut t: i32 = 0;
        if self.x == 0 || self.x == self.width.wrapping_sub(1) {
            t = t.wrapping_add(1);
        }
        if self.y == 0 || self.y == self.height.wrapping_sub(1) {
            t = t.wrapping_add(1);
        }
        if self.z == 0 || self.z == self.depth.wrapping_sub(1) {
            t = t.wrapping_add(1);
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_block() {
        let mut c = Cursor3D::new(5, 6, 7, 5, 6, 7);
        assert!(c.advance());
        assert_eq!(c.next_x(), 5);
        assert_eq!(c.next_y(), 6);
        assert_eq!(c.next_z(), 7);
        assert_eq!(c.get_next_type(), TYPE_CORNER);
        assert!(!c.advance());
    }

    #[test]
    fn iterates_all_positions() {
        let mut c = Cursor3D::new(0, 0, 0, 1, 1, 1);
        let mut positions = Vec::new();
        while c.advance() {
            positions.push((c.next_x(), c.next_y(), c.next_z()));
        }
        assert_eq!(positions.len(), 8);
        assert_eq!(positions[0], (0, 0, 0));
        assert_eq!(positions[1], (1, 0, 0));
        assert_eq!(positions[2], (0, 1, 0));
        assert_eq!(positions[3], (1, 1, 0));
        assert_eq!(positions[7], (1, 1, 1));
    }

    #[test]
    fn type_classification_3x3x3() {
        let mut c = Cursor3D::new(0, 0, 0, 2, 2, 2);
        let mut counts = [0i32; 4];
        while c.advance() {
            let t = c.get_next_type() as usize;
            counts[t] += 1;
        }
        assert_eq!(counts[TYPE_INSIDE as usize], 1);
        assert_eq!(counts[TYPE_FACE as usize], 6);
        assert_eq!(counts[TYPE_EDGE as usize], 12);
        assert_eq!(counts[TYPE_CORNER as usize], 8);
    }

    #[test]
    fn negative_origin() {
        let mut c = Cursor3D::new(-1, -2, -3, 0, 0, -1);
        let mut positions = Vec::new();
        while c.advance() {
            positions.push((c.next_x(), c.next_y(), c.next_z()));
        }
        assert_eq!(positions.len(), 18); // 2 * 3 * 3
        assert_eq!(positions[0], (-1, -2, -3));
        assert!(positions.contains(&(0, 0, -1)));
    }

    #[test]
    fn advance_after_end_returns_false() {
        let mut c = Cursor3D::new(0, 0, 0, 0, 0, 0);
        assert!(c.advance());
        assert!(!c.advance());
        assert!(!c.advance());
        assert_eq!(c.next_x(), 0);
    }
}
