/// Minimal self-contained tag type descriptor.
// JAVA: net.minecraft.nbt.TagType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagType {
    pub id: i32,
    pub name: &'static str,
    pub valid: bool,
}

impl TagType {
    // JAVA: net.minecraft.nbt.TagType#createInvalid
    pub fn create_invalid(id: i32) -> Self {
        Self {
            id,
            name: "INVALID",
            valid: false,
        }
    }
}

// JAVA: net.minecraft.nbt.TagTypes#TYPES
const TYPES: [TagType; 13] = [
    TagType {
        id: 0,
        name: "END",
        valid: true,
    },
    TagType {
        id: 1,
        name: "BYTE",
        valid: true,
    },
    TagType {
        id: 2,
        name: "SHORT",
        valid: true,
    },
    TagType {
        id: 3,
        name: "INT",
        valid: true,
    },
    TagType {
        id: 4,
        name: "LONG",
        valid: true,
    },
    TagType {
        id: 5,
        name: "FLOAT",
        valid: true,
    },
    TagType {
        id: 6,
        name: "DOUBLE",
        valid: true,
    },
    TagType {
        id: 7,
        name: "BYTE[]",
        valid: true,
    },
    TagType {
        id: 8,
        name: "STRING",
        valid: true,
    },
    TagType {
        id: 9,
        name: "LIST",
        valid: true,
    },
    TagType {
        id: 10,
        name: "COMPOUND",
        valid: true,
    },
    TagType {
        id: 11,
        name: "INT[]",
        valid: true,
    },
    TagType {
        id: 12,
        name: "LONG[]",
        valid: true,
    },
];

// JAVA: net.minecraft.nbt.TagTypes#getType
pub fn get_type(id: i32) -> TagType {
    if id >= 0 && (id as usize) < TYPES.len() {
        TYPES[id as usize]
    } else {
        TagType::create_invalid(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_type_valid_range() {
        let t0 = get_type(0);
        assert_eq!(t0.id, 0);
        assert_eq!(t0.name, "END");
        assert!(t0.valid);

        let t12 = get_type(12);
        assert_eq!(t12.id, 12);
        assert_eq!(t12.name, "LONG[]");
        assert!(t12.valid);
    }

    #[test]
    fn get_type_negative_returns_invalid() {
        let t = get_type(-1);
        assert_eq!(t.id, -1);
        assert_eq!(t.name, "INVALID");
        assert!(!t.valid);

        let tmin = get_type(i32::MIN);
        assert_eq!(tmin.id, i32::MIN);
        assert!(!tmin.valid);
    }

    #[test]
    fn get_type_out_of_bounds_returns_invalid() {
        let t = get_type(13);
        assert_eq!(t.id, 13);
        assert!(!t.valid);
        let tmax = get_type(i32::MAX);
        assert_eq!(tmax.id, i32::MAX);
        assert!(!tmax.valid);
    }

    #[test]
    fn get_type_matches_all_known_ids() {
        let expected = [
            "END", "BYTE", "SHORT", "INT", "LONG", "FLOAT", "DOUBLE", "BYTE[]", "STRING", "LIST",
            "COMPOUND", "INT[]", "LONG[]",
        ];
        for (i, name) in expected.iter().enumerate() {
            let t = get_type(i as i32);
            assert_eq!(t.id, i as i32);
            assert_eq!(t.name, *name);
            assert!(t.valid);
        }
    }
}
