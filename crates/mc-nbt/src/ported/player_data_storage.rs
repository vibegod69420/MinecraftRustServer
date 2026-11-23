use crate::NbtTag;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameAndId {
    pub uuid: String,
    pub player_name: String,
}

impl NameAndId {
    // JAVA: net.minecraft.server.players.NameAndId
    pub fn new(uuid: impl Into<String>, player_name: impl Into<String>) -> Self {
        Self {
            uuid: uuid.into(),
            player_name: player_name.into(),
        }
    }

    // JAVA: net.minecraft.server.players.NameAndId#id
    pub fn id(&self) -> &str {
        &self.uuid
    }

    // JAVA: net.minecraft.server.players.NameAndId#name
    pub fn name(&self) -> &str {
        &self.player_name
    }
}

#[derive(Debug, Default, Clone)]
pub struct DataFixer;

pub struct ScopedCollector;
impl Drop for ScopedCollector {
    fn drop(&mut self) {}
}

pub struct ProblemReporter;

impl ProblemReporter {
    // JAVA: net.minecraft.util.ProblemReporter$ScopedCollector
    pub fn scoped_collector(_path: String) -> ScopedCollector {
        ScopedCollector
    }
}

pub struct TagValueOutput {
    result: NbtTag,
}

impl TagValueOutput {
    // JAVA: net.minecraft.world.level.storage.TagValueOutput#createWithContext
    pub fn create_with_context(_reporter: &ScopedCollector, _registry_access: ()) -> Self {
        Self {
            result: NbtTag::Compound(Vec::new()),
        }
    }

    // JAVA: net.minecraft.world.level.storage.TagValueOutput#buildResult
    pub fn build_result(self) -> NbtTag {
        self.result
    }

    pub fn set_result(&mut self, tag: NbtTag) {
        self.result = tag;
    }
}

pub trait PlayerLike {
    fn problem_path(&self) -> String;
    fn registry_access(&self) -> ();
    fn save_without_id(&self, output: &mut TagValueOutput);
    fn string_uuid(&self) -> String;
    fn plain_text_name(&self) -> String;
}

#[derive(Clone, Copy)]
pub enum DataFixTypes {
    Player,
}

impl DataFixTypes {
    // JAVA: net.minecraft.util.datafix.DataFixTypes#updateToCurrentVersion
    pub fn update_to_current_version(self, _fixer: &DataFixer, tag: NbtTag, _version: i32) -> NbtTag {
        tag
    }
}

// JAVA: net.minecraft.nbt.NbtUtils#getDataVersion
pub fn get_data_version(tag: &NbtTag) -> i32 {
    match tag.compound_get("DataVersion") {
        Some(NbtTag::Int(v)) => *v,
        _ => 0,
    }
}

fn write_mutf8<W: Write>(w: &mut W, s: &str) -> io::Result<()> {
    let mut bytes: Vec<u8> = Vec::new();
    for c in s.chars() {
        let v = c as u32;
        if v >= 0x0001 && v <= 0x007F {
            bytes.push(v as u8);
        } else if v == 0 || (v >= 0x0080 && v <= 0x07FF) {
            bytes.push((0xC0 | ((v >> 6) & 0x1F)) as u8);
            bytes.push((0x80 | (v & 0x3F)) as u8);
        } else if v <= 0xFFFF {
            bytes.push((0xE0 | ((v >> 12) & 0x0F)) as u8);
            bytes.push((0x80 | ((v >> 6) & 0x3F)) as u8);
            bytes.push((0x80 | (v & 0x3F)) as u8);
        } else {
            let u = v - 0x10000;
            let hi = 0xD800 + ((u >> 10) & 0x3FF);
            let lo = 0xDC00 + (u & 0x3FF);
            for sv in [hi, lo] {
                bytes.push((0xE0 | ((sv >> 12) & 0x0F)) as u8);
                bytes.push((0x80 | ((sv >> 6) & 0x3F)) as u8);
                bytes.push((0x80 | (sv & 0x3F)) as u8);
            }
        }
    }
    if bytes.len() > u16::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "string too long"));
    }
    w.write_all(&(bytes.len() as u16).to_be_bytes())?;
    w.write_all(&bytes)
}

fn read_mutf8<R: Read>(r: &mut R) -> io::Result<String> {
    let mut len_buf = [0u8; 2];
    r.read_exact(&mut len_buf)?;
    let len = u16::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    let mut chars: Vec<char> = Vec::new();
    let mut i = 0usize;
    while i < buf.len() {
        let b = buf[i] as u32;
        if b & 0x80 == 0 {
            chars.push(char::from_u32(b).unwrap_or('\u{FFFD}'));
            i += 1;
        } else if b & 0xE0 == 0xC0 {
            if i + 1 >= buf.len() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid mutf8"));
            }
            let b2 = buf[i + 1] as u32;
            let c = ((b & 0x1F) << 6) | (b2 & 0x3F);
            chars.push(char::from_u32(c).unwrap_or('\u{FFFD}'));
            i += 2;
        } else if b & 0xF0 == 0xE0 {
            if i + 2 >= buf.len() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid mutf8"));
            }
            let b2 = buf[i + 1] as u32;
            let b3 = buf[i + 2] as u32;
            let c = ((b & 0x0F) << 12) | ((b2 & 0x3F) << 6) | (b3 & 0x3F);
            if (0xD800..=0xDBFF).contains(&c) && i + 6 <= buf.len() && (buf[i + 3] & 0xF0) == 0xE0 {
                let b4 = buf[i + 3] as u32;
                let b5 = buf[i + 4] as u32;
                let b6 = buf[i + 5] as u32;
                let lo = ((b4 & 0x0F) << 12) | ((b5 & 0x3F) << 6) | (b6 & 0x3F);
                if (0xDC00..=0xDFFF).contains(&lo) {
                    let cp = 0x10000 + (((c - 0xD800) << 10) | (lo - 0xDC00));
                    chars.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                    i += 6;
                    continue;
                }
            }
            chars.push(char::from_u32(c).unwrap_or('\u{FFFD}'));
            i += 3;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid mutf8"));
        }
    }
    Ok(chars.into_iter().collect())
}

fn read_i32<R: Read>(r: &mut R) -> io::Result<i32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(i32::from_be_bytes(b))
}

fn write_payload<W: Write>(w: &mut W, tag: &NbtTag) -> io::Result<()> {
    match tag {
        NbtTag::Byte(v) => w.write_all(&[*v as u8]),
        NbtTag::Short(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::Int(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::Long(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::Float(v) => w.write_all(&v.to_bits().to_be_bytes()),
        NbtTag::Double(v) => w.write_all(&v.to_bits().to_be_bytes()),
        NbtTag::ByteArray(v) => {
            w.write_all(&(v.len() as i32).to_be_bytes())?;
            for b in v {
                w.write_all(&[*b as u8])?;
            }
            Ok(())
        }
        NbtTag::String(s) => write_mutf8(w, s),
        NbtTag::List { elem_type, items } => {
            w.write_all(&[*elem_type])?;
            w.write_all(&(items.len() as i32).to_be_bytes())?;
            for item in items {
                write_payload(w, item)?;
            }
            Ok(())
        }
        NbtTag::Compound(entries) => {
            for (k, v) in entries {
                write_named_tag(w, k, v)?;
            }
            w.write_all(&[0u8])
        }
        NbtTag::IntArray(v) => {
            w.write_all(&(v.len() as i32).to_be_bytes())?;
            for i in v {
                w.write_all(&i.to_be_bytes())?;
            }
            Ok(())
        }
        NbtTag::LongArray(v) => {
            w.write_all(&(v.len() as i32).to_be_bytes())?;
            for i in v {
                w.write_all(&i.to_be_bytes())?;
            }
            Ok(())
        }
    }
}

fn read_payload<R: Read>(r: &mut R, id: u8) -> io::Result<NbtTag> {
    Ok(match id {
        1 => {
            let mut b = [0u8; 1];
            r.read_exact(&mut b)?;
            NbtTag::Byte(b[0] as i8)
        }
        2 => {
            let mut b = [0u8; 2];
            r.read_exact(&mut b)?;
            NbtTag::Short(i16::from_be_bytes(b))
        }
        3 => NbtTag::Int(read_i32(r)?),
        4 => {
            let mut b = [0u8; 8];
            r.read_exact(&mut b)?;
            NbtTag::Long(i64::from_be_bytes(b))
        }
        5 => {
            let mut b = [0u8; 4];
            r.read_exact(&mut b)?;
            NbtTag::Float(f32::from_bits(u32::from_be_bytes(b)))
        }
        6 => {
            let mut b = [0u8; 8];
            r.read_exact(&mut b)?;
            NbtTag::Double(f64::from_bits(u64::from_be_bytes(b)))
        }
        7 => {
            let len = read_i32(r)?;
            if len < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "negative length"));
            }
            let mut buf = vec![0u8; len as usize];
            r.read_exact(&mut buf)?;
            NbtTag::ByteArray(buf.into_iter().map(|b| b as i8).collect())
        }
        8 => NbtTag::String(read_mutf8(r)?),
        9 => {
            let mut t = [0u8; 1];
            r.read_exact(&mut t)?;
            let len = read_i32(r)?;
            if len < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "negative length"));
            }
            let mut items = Vec::with_capacity(len as usize);
            for _ in 0..len {
                items.push(read_payload(r, t[0])?);
            }
            NbtTag::List {
                elem_type: t[0],
                items,
            }
        }
        10 => {
            let mut entries = Vec::new();
            loop {
                let mut idb = [0u8; 1];
                r.read_exact(&mut idb)?;
                if idb[0] == 0 {
                    break;
                }
                let name = read_mutf8(r)?;
                let tag = read_payload(r, idb[0])?;
                entries.push((name, tag));
            }
            NbtTag::Compound(entries)
        }
        11 => {
            let len = read_i32(r)?;
            if len < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "negative length"));
            }
            let mut v = Vec::with_capacity(len as usize);
            for _ in 0..len {
                v.push(read_i32(r)?);
            }
            NbtTag::IntArray(v)
        }
        12 => {
            let len = read_i32(r)?;
            if len < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "negative length"));
            }
            let mut v = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let mut b = [0u8; 8];
                r.read_exact(&mut b)?;
                v.push(i64::from_be_bytes(b));
            }
            NbtTag::LongArray(v)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown tag id {}", id),
            ))
        }
    })
}

fn write_named_tag<W: Write>(w: &mut W, name: &str, tag: &NbtTag) -> io::Result<()> {
    let id = tag.id();
    w.write_all(&[id])?;
    if id != 0 {
        write_mutf8(w, name)?;
        write_payload(w, tag)?;
    }
    Ok(())
}

fn read_named_tag<R: Read>(r: &mut R) -> io::Result<NbtTag> {
    let mut id_buf = [0u8; 1];
    r.read_exact(&mut id_buf)?;
    let id = id_buf[0];
    if id == 0 {
        return Ok(NbtTag::Compound(Vec::new()));
    }
    let _name = read_mutf8(r)?;
    read_payload(r, id)
}

// JAVA: net.minecraft.nbt.NbtIo#writeCompressed
pub fn write_compressed_nbt(tag: &NbtTag, path: &Path) -> io::Result<()> {
    let file = File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    write_named_tag(&mut encoder, "", tag)?;
    encoder.finish()?;
    Ok(())
}

// JAVA: net.minecraft.nbt.NbtIo#readCompressed
pub fn read_compressed_nbt(path: &Path) -> io::Result<NbtTag> {
    let file = File::open(path)?;
    let mut decoder = GzDecoder::new(file);
    read_named_tag(&mut decoder)
}

// JAVA: net.minecraft.util.Util#safeReplaceFile
fn safe_replace_file(target: &Path, source: &Path, backup: &Path) -> io::Result<()> {
    if backup.exists() {
        fs::remove_file(backup)?;
    }
    if target.exists() {
        fs::rename(target, backup)?;
    }
    fs::rename(source, target)
}

fn timestamp_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

// JAVA: net.minecraft.world.level.storage.PlayerDataStorage
pub struct PlayerDataStorage {
    player_dir: PathBuf,
    fixer_upper: DataFixer,
}

impl PlayerDataStorage {
    // JAVA: net.minecraft.world.level.storage.PlayerDataStorage#<init>
    pub fn new(player_dir: PathBuf, fixer_upper: DataFixer) -> Self {
        let _ = fs::create_dir_all(&player_dir);
        Self {
            player_dir,
            fixer_upper,
        }
    }

    // JAVA: net.minecraft.world.level.storage.PlayerDataStorage#save
    pub fn save<P: PlayerLike>(&self, player: &P) {
        let result = (|| -> io::Result<()> {
            let reporter = ProblemReporter::scoped_collector(player.problem_path());
            let mut output =
                TagValueOutput::create_with_context(&reporter, player.registry_access());
            player.save_without_id(&mut output);
            let uuid = player.string_uuid();
            let mut tmp_path = self.player_dir.join(format!("{}-.dat.tmp", uuid));
            let mut n: u32 = 0;
            while tmp_path.exists() {
                n = n.wrapping_add(1);
                tmp_path = self.player_dir.join(format!("{}-{}.dat.tmp", uuid, n));
            }
            let tag = output.build_result();
            write_compressed_nbt(&tag, &tmp_path)?;
            let target = self.player_dir.join(format!("{}.dat", uuid));
            let backup = self.player_dir.join(format!("{}.dat_old", uuid));
            safe_replace_file(&target, &tmp_path, &backup)?;
            Ok(())
        })();
        if result.is_err() {
            eprintln!("Failed to save player data for {}", player.plain_text_name());
        }
    }

    // JAVA: net.minecraft.world.level.storage.PlayerDataStorage#backup
    fn backup(&self, name_and_id: &NameAndId, suffix: &str) {
        let id = name_and_id.id();
        let src = self.player_dir.join(format!("{}{}", id, suffix));
        let ts = timestamp_string();
        let dst = self
            .player_dir
            .join(format!("{}_corrupted_{}{}", id, ts, suffix));
        if src.is_file() {
            if let Err(e) = fs::copy(&src, &dst) {
                eprintln!(
                    "Failed to copy the player.dat file for {}: {}",
                    name_and_id.name(),
                    e
                );
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.PlayerDataStorage#load(NameAndId, String)
    fn load_with_suffix(&self, name_and_id: &NameAndId, suffix: &str) -> Option<NbtTag> {
        let path = self.player_dir.join(format!("{}{}", name_and_id.id(), suffix));
        if path.is_file() {
            match read_compressed_nbt(&path) {
                Ok(tag) => return Some(tag),
                Err(_) => {
                    eprintln!("Failed to load player data for {}", name_and_id.name());
                }
            }
        }
        None
    }

    // JAVA: net.minecraft.world.level.storage.PlayerDataStorage#load(NameAndId)
    pub fn load(&self, name_and_id: &NameAndId) -> Option<NbtTag> {
        let main = self.load_with_suffix(name_and_id, ".dat");
        if main.is_none() {
            self.backup(name_and_id, ".dat");
        }
        main.or_else(|| self.load_with_suffix(name_and_id, ".dat_old"))
            .map(|tag| {
                let version = get_data_version(&tag);
                DataFixTypes::Player.update_to_current_version(&self.fixer_upper, tag, version)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    struct MockPlayer {
        uuid: String,
        name: String,
        tag: NbtTag,
    }

    impl PlayerLike for MockPlayer {
        fn problem_path(&self) -> String {
            self.name.clone()
        }
        fn registry_access(&self) -> () {}
        fn save_without_id(&self, output: &mut TagValueOutput) {
            output.set_result(self.tag.clone());
        }
        fn string_uuid(&self) -> String {
            self.uuid.clone()
        }
        fn plain_text_name(&self) -> String {
            self.name.clone()
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut p = std::env::temp_dir();
        p.push(format!(
            "mc_nbt_pds_{}_{}_{}",
            name,
            std::process::id(),
            n
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = temp_dir("roundtrip");
        let storage = PlayerDataStorage::new(dir.clone(), DataFixer);
        let tag = NbtTag::Compound(vec![
            ("Health".to_string(), NbtTag::Int(20)),
            ("Name".to_string(), NbtTag::String("Alice".to_string())),
        ]);
        let player = MockPlayer {
            uuid: "u1".into(),
            name: "Alice".into(),
            tag: tag.clone(),
        };
        storage.save(&player);
        let loaded = storage.load(&NameAndId::new("u1", "Alice")).unwrap();
        assert_eq!(loaded, tag);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_falls_back_to_dat_old_and_backups_corrupt_dat() {
        let dir = temp_dir("fallback");
        let storage = PlayerDataStorage::new(dir.clone(), DataFixer);
        let tag = NbtTag::Compound(vec![("X".to_string(), NbtTag::Int(1))]);
        write_compressed_nbt(&tag, &dir.join("u2.dat_old")).unwrap();
        fs::write(dir.join("u2.dat"), b"not nbt").unwrap();

        let loaded = storage.load(&NameAndId::new("u2", "Bob"));
        assert_eq!(loaded, Some(tag));

        let has_backup = fs::read_dir(&dir).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .contains("_corrupted_")
        });
        assert!(has_backup);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_returns_none_when_missing() {
        let dir = temp_dir("missing");
        let storage = PlayerDataStorage::new(dir.clone(), DataFixer);
        let loaded = storage.load(&NameAndId::new("missing-uuid", "Nobody"));
        assert!(loaded.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_data_version_reads_int_or_zero() {
        let tag = NbtTag::Compound(vec![("DataVersion".to_string(), NbtTag::Int(4321))]);
        assert_eq!(get_data_version(&tag), 4321);
        let empty = NbtTag::Compound(vec![]);
        assert_eq!(get_data_version(&empty), 0);
        let wrong = NbtTag::Compound(vec![("DataVersion".to_string(), NbtTag::String("x".into()))]);
        assert_eq!(get_data_version(&wrong), 0);
    }

    #[test]
    fn compressed_nbt_roundtrip_preserves_float_bits_and_negative() {
        let dir = temp_dir("nbt_bits");
        let path = dir.join("test.dat");
        let tag = NbtTag::Compound(vec![
            ("f".to_string(), NbtTag::Float(f32::NEG_INFINITY)),
            ("d".to_string(), NbtTag::Double(f64::NAN)),
            ("b".to_string(), NbtTag::Byte(-1)),
            ("arr".to_string(), NbtTag::IntArray(vec![-1, 0, i32::MIN])),
        ]);
        write_compressed_nbt(&tag, &path).unwrap();
        let loaded = read_compressed_nbt(&path).unwrap();
        match (&tag, &loaded) {
            (NbtTag::Compound(a), NbtTag::Compound(b)) => {
                assert_eq!(a.len(), b.len());
                if let (Some(NbtTag::Float(f1)), Some(NbtTag::Float(f2))) =
                    (loaded.compound_get("f"), Some(&NbtTag::Float(f32::NEG_INFINITY)))
                {
                    assert_eq!(f1.to_bits(), f2.to_bits());
                }
                if let Some(NbtTag::Double(d)) = loaded.compound_get("d") {
                    assert!(d.is_nan());
                }
                assert_eq!(loaded.compound_get("b"), Some(&NbtTag::Byte(-1)));
                assert_eq!(
                    loaded.compound_get("arr"),
                    Some(&NbtTag::IntArray(vec![-1, 0, i32::MIN]))
                );
            }
            _ => panic!("expected compound"),
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
