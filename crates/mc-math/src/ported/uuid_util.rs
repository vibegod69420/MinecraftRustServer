use std::fmt;

pub const UUID_BYTES: usize = 16;

/// Minimal UUID representation matching `java.util.UUID`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    most_significant_bits: i64,
    least_significant_bits: i64,
}

impl Uuid {
    pub fn new(most_significant_bits: i64, least_significant_bits: i64) -> Self {
        Self {
            most_significant_bits,
            least_significant_bits,
        }
    }

    pub fn get_most_significant_bits(&self) -> i64 {
        self.most_significant_bits
    }

    pub fn get_least_significant_bits(&self) -> i64 {
        self.least_significant_bits
    }

    pub fn version(&self) -> i32 {
        ((self.most_significant_bits >> 12) & 0x0f) as i32
    }

    pub fn variant(&self) -> i32 {
        (((self.least_significant_bits as u64) >> 62) & 0x3) as i32
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let most = self.most_significant_bits as u64;
        let least = self.least_significant_bits as u64;
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (most >> 32) as u32,
            (most >> 16) as u16 as u32,
            most as u16 as u32,
            (least >> 48) as u16 as u32,
            least & 0x0000_ffff_ffff_ffffu64,
        )
    }
}

/// Minimal stand-in for `com.mojang.authlib.GameProfile`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
}

// JAVA: net.minecraft.core.UUIDUtil#uuidFromIntArray
pub fn uuid_from_int_array(arr: [i32; 4]) -> Uuid {
    let most = ((arr[0] as i64) << 32) | (arr[1] as u32 as i64);
    let least = ((arr[2] as i64) << 32) | (arr[3] as u32 as i64);
    Uuid::new(most, least)
}

// JAVA: net.minecraft.core.UUIDUtil#uuidToIntArray
pub fn uuid_to_int_array(uuid: &Uuid) -> [i32; 4] {
    let most = uuid.get_most_significant_bits();
    let least = uuid.get_least_significant_bits();
    least_most_to_int_array(most, least)
}

// JAVA: net.minecraft.core.UUIDUtil#leastMostToIntArray
fn least_most_to_int_array(most: i64, least: i64) -> [i32; 4] {
    [
        (most >> 32) as i32,
        most as i32,
        (least >> 32) as i32,
        least as i32,
    ]
}

// JAVA: net.minecraft.core.UUIDUtil#uuidToByteArray
pub fn uuid_to_byte_array(uuid: &Uuid) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&uuid.get_most_significant_bits().to_be_bytes());
    bytes[8..16].copy_from_slice(&uuid.get_least_significant_bits().to_be_bytes());
    bytes
}

// JAVA: net.minecraft.core.UUIDUtil#readUUID
pub fn read_uuid(ints: &[i32]) -> Result<Uuid, String> {
    if ints.len() != 4 {
        return Err(format!(
            "Could not read UUID. Expected int-array of length 4, got {}.",
            ints.len()
        ));
    }
    Ok(uuid_from_int_array([ints[0], ints[1], ints[2], ints[3]]))
}

// JAVA: net.minecraft.core.UUIDUtil#createOfflinePlayerUUID
pub fn create_offline_player_uuid(name: &str) -> Uuid {
    let input = format!("OfflinePlayer:{name}");
    let mut hash = md5(input.as_bytes());
    hash[6] = (hash[6] & 0x0f) | 0x30;
    hash[8] = (hash[8] & 0x3f) | 0x80;
    uuid_from_bytes(hash)
}

// JAVA: net.minecraft.core.UUIDUtil#createOfflineProfile
pub fn create_offline_profile(name: &str) -> GameProfile {
    let id = create_offline_player_uuid(name);
    GameProfile {
        id,
        name: name.to_owned(),
    }
}

fn uuid_from_bytes(bytes: [u8; 16]) -> Uuid {
    let most = i64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    let least = i64::from_be_bytes([
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]);
    Uuid::new(most, least)
}

fn md5(input: &[u8]) -> [u8; 16] {
    let mut a0: u32 = 0x6745_2301;
    let mut b0: u32 = 0xefcd_ab89;
    let mut c0: u32 = 0x98ba_dcfe;
    let mut d0: u32 = 0x1032_5476;

    let orig_len_bits = (input.len() as u64).wrapping_mul(8);

    let mut msg = input.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&orig_len_bits.to_le_bytes());

    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76a_a478,
        0xe8c7_b756,
        0x2420_70db,
        0xc1bd_ceee,
        0xf57c_0faf,
        0x4787_c62a,
        0xa830_4613,
        0xfd46_9501,
        0x6980_98d8,
        0x8b44_f7af,
        0xffff_5bb1,
        0x895c_d7be,
        0x6b90_1122,
        0xfd98_7193,
        0xa679_438e,
        0x49b4_0821,
        0xf61e_2562,
        0xc040_b340,
        0x265e_5a51,
        0xe9b6_c7aa,
        0xd62f_105d,
        0x0244_1453,
        0xd8a1_e681,
        0xe7d3_fbc8,
        0x21e1_cde6,
        0xc337_07d6,
        0xf4d5_0d87,
        0x455a_14ed,
        0xa9e3_e905,
        0xfcef_a3f8,
        0x676f_02d9,
        0x8d2a_4c8a,
        0xfffa_3942,
        0x8771_f681,
        0x6d9d_6122,
        0xfde5_380c,
        0xa4be_ea44,
        0x4bde_cfa9,
        0xf6bb_4b60,
        0xbebf_bc70,
        0x289b_7ec6,
        0xeaa1_27fa,
        0xd4ef_3085,
        0x0488_1d05,
        0xd9d4_d039,
        0xe6db_99e5,
        0x1fa2_7cf8,
        0xc4ac_5665,
        0xf429_2244,
        0x432a_ff97,
        0xab94_23a7,
        0xfc93_a039,
        0x655b_59c3,
        0x8f0c_cc92,
        0xffef_f47d,
        0x8584_5dd1,
        0x6fa8_7e4f,
        0xfe2c_e6e0,
        0xa301_4314,
        0x4e08_11a1,
        0xf753_7e82,
        0xbd3a_f235,
        0x2ad7_d2bb,
        0xeb86_d391,
    ];

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, word) in m.iter_mut().enumerate() {
            let j = i * 4;
            *word = u32::from_le_bytes([chunk[j], chunk[j + 1], chunk[j + 2], chunk[j + 3]]);
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64usize {
            let (f, g): (u32, usize) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5usize.wrapping_mul(i).wrapping_add(1)) % 16),
                32..=47 => (b ^ c ^ d, (3usize.wrapping_mul(i).wrapping_add(5)) % 16),
                48..=63 => (c ^ (b | (!d)), (7usize.wrapping_mul(i)) % 16),
                _ => unreachable!(),
            };
            let temp = d;
            d = c;
            c = b;
            let f = f
                .wrapping_add(a)
                .wrapping_add(K[i])
                .wrapping_add(m[g]);
            b = b.wrapping_add(f.rotate_left(S[i]));
            a = temp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a0.to_le_bytes());
    out[4..8].copy_from_slice(&b0.to_le_bytes());
    out[8..12].copy_from_slice(&c0.to_le_bytes());
    out[12..16].copy_from_slice(&d0.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_from_int_array_round_trip_and_negative() {
        let ints = [-1, -2, i32::MIN, i32::MAX];
        let uuid = uuid_from_int_array(ints);
        assert_eq!(uuid_to_int_array(&uuid), ints);
        let bytes = uuid_to_byte_array(&uuid);
        let most = i64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let least = i64::from_be_bytes(bytes[8..16].try_into().unwrap());
        assert_eq!(Uuid::new(most, least), uuid);
    }

    #[test]
    fn uuid_to_byte_array_big_endian_layout() {
        let uuid = Uuid::new(0x0102_0304_0506_0708i64, 0xFEFD_FCFB_FAF9_F8F9u64 as i64);
        assert_eq!(
            uuid_to_byte_array(&uuid),
            [1, 2, 3, 4, 5, 6, 7, 8, 254, 253, 252, 251, 250, 249, 248, 249]
        );
    }

    #[test]
    fn create_offline_player_uuid_known_vector_and_version_bits() {
        let name = "Notch";
        let mut hash = md5(format!("OfflinePlayer:{name}").as_bytes());
        hash[6] = (hash[6] & 0x0f) | 0x30;
        hash[8] = (hash[8] & 0x3f) | 0x80;
        let expected = uuid_from_bytes(hash);
        let uuid = create_offline_player_uuid(name);
        assert_eq!(uuid, expected);
        assert_eq!(uuid.version(), 3);
        assert_eq!(uuid.variant(), 2);
        assert_eq!(uuid_to_int_array(&uuid), uuid_to_int_array(&expected));
    }

    #[test]
    fn md5_known_vectors() {
        assert_eq!(
            md5(b""),
            [
                0xd4, 0x1d, 0x8c, 0xd9, 0x8f, 0x00, 0xb2, 0x04, 0xe9, 0x80, 0x09, 0x98, 0xec, 0xf8,
                0x42, 0x7e
            ]
        );
        assert_eq!(
            md5(b"abc"),
            [
                0x90, 0x01, 0x50, 0x98, 0x3c, 0xd2, 0x4f, 0xb0, 0xd6, 0x96, 0x3f, 0x7d, 0x28, 0xe1,
                0x7f, 0x72
            ]
        );
    }

    #[test]
    fn read_uuid_rejects_wrong_length_and_round_trips() {
        let err = read_uuid(&[1, 2, 3]).unwrap_err();
        assert_eq!(
            err,
            "Could not read UUID. Expected int-array of length 4, got 3."
        );
        let err = read_uuid(&[1, 2, 3, 4, 5]).unwrap_err();
        assert_eq!(
            err,
            "Could not read UUID. Expected int-array of length 4, got 5."
        );
        let ints = [i32::MIN, -7, 0, i32::MAX];
        let ok = read_uuid(&ints).unwrap();
        assert_eq!(uuid_to_int_array(&ok), ints);
    }
}
