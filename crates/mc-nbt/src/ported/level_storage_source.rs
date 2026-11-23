// === FILE: level_storage_source.rs ===
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::{NbtError, NbtTag};

pub const TAG_DATA: &str = "Data";
pub const ALLOWED_SYMLINKS_CONFIG_NAME: &str = "allowed_symlinks.txt";
const DISK_SPACE_WARNING_THRESHOLD: i64 = 67_108_864;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub key: String,
}

impl Component {
    pub fn translatable(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct LevelStorageException {
    pub message: Component,
}

impl std::fmt::Display for LevelStorageException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message.key)
    }
}

impl std::error::Error for LevelStorageException {}

#[derive(Debug, Clone)]
pub struct ForbiddenSymlinkInfo {
    pub link: PathBuf,
}

impl ForbiddenSymlinkInfo {
    pub fn new(link: PathBuf) -> Self {
        Self { link }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PathAllowList;

impl PathAllowList {
    pub fn read_plain<R: BufRead>(_reader: R) -> Result<Self, io::Error> {
        Ok(Self)
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryValidator {
    no_symlinks: bool,
}

impl DirectoryValidator {
    pub fn new(_allow: PathAllowList) -> Self {
        Self {
            no_symlinks: false,
        }
    }

    pub fn no_symlinks() -> Self {
        Self { no_symlinks: true }
    }

    pub fn validate_symlink(&self, path: &Path) -> Vec<ForbiddenSymlinkInfo> {
        if self.no_symlinks {
            vec![ForbiddenSymlinkInfo::new(path.to_path_buf())]
        } else {
            Vec::new()
        }
    }

    pub fn validate_directory(&self, path: &Path, _recursive: bool) -> Vec<ForbiddenSymlinkInfo> {
        if self.no_symlinks {
            match fs::symlink_metadata(path) {
                Ok(m) if m.file_type().is_symlink() => {
                    vec![ForbiddenSymlinkInfo::new(path.to_path_buf())]
                }
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct ContentValidationException {
    pub path: PathBuf,
    pub infos: Vec<ForbiddenSymlinkInfo>,
}

impl std::fmt::Display for ContentValidationException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Content validation failed for {} ({} symlink issues)",
            self.path.display(),
            self.infos.len()
        )
    }
}

impl std::error::Error for ContentValidationException {}

#[derive(Debug)]
pub struct DirectoryLock {
    path: PathBuf,
    valid: bool,
}

impl DirectoryLock {
    pub fn create(path: &Path) -> io::Result<Self> {
        let lock_path = path.join(LevelResource::LockFile.get_id());
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let _ = File::create(&lock_path)?;
        Ok(Self {
            path: lock_path,
            valid: true,
        })
    }

    pub fn is_locked(path: &Path) -> io::Result<bool> {
        let lock_path = path.join(LevelResource::LockFile.get_id());
        Ok(lock_path.exists())
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn close(&mut self) -> io::Result<()> {
        self.valid = false;
        let _ = self.path;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct DataFixer;

#[derive(Debug, Clone)]
pub struct Dynamic {
    pub value: NbtTag,
}

impl Dynamic {
    pub fn new(value: NbtTag) -> Self {
        Self { value }
    }

    pub fn get(&self, key: &str) -> Option<Dynamic> {
        match &self.value {
            NbtTag::Compound(entries) => entries
                .iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, v)| Dynamic::new(v.clone())),
            _ => None,
        }
    }

    pub fn get_or_empty_map(&self, key: &str) -> Dynamic {
        self.get(key)
            .unwrap_or_else(|| Dynamic::new(NbtTag::Compound(Vec::new())))
    }

    pub fn as_string_result(&self) -> Option<String> {
        match &self.value {
            NbtTag::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_stream(&self) -> Vec<Dynamic> {
        match &self.value {
            NbtTag::List { items, .. } => items.iter().cloned().map(Dynamic::new).collect(),
            _ => Vec::new(),
        }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(mut self, key: &str, f: F) -> Self {
        if let NbtTag::Compound(ref mut entries) = self.value {
            if let Some(pos) = entries.iter().rposition(|(k, _)| k == key) {
                let old = entries[pos].1.clone();
                entries[pos].1 = f(Dynamic::new(old)).value;
            }
        }
        self
    }
}

pub struct NbtUtils;

impl NbtUtils {
    pub fn get_data_version(tag: &NbtTag) -> i32 {
        match tag.compound_get("DataVersion") {
            Some(NbtTag::Int(v)) => *v,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DataFixTypes {
    Level,
    Player,
    WorldGenSettings,
    LevelSummary,
}

impl DataFixTypes {
    pub fn update_to_current_version(
        &self,
        _fixer: &DataFixer,
        data: Dynamic,
        _version: i32,
    ) -> Dynamic {
        data
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorldDataConfiguration;

#[derive(Debug, Clone, Default)]
pub struct PackRepository;

#[derive(Debug, Clone)]
pub struct PackConfig {
    pub pack_repository: PackRepository,
    pub data_configuration: WorldDataConfiguration,
    pub safe_mode: bool,
    pub stable: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Registry<T>(std::marker::PhantomData<T>);

#[derive(Debug, Clone, Default)]
pub struct HolderLookupProvider;

#[derive(Debug, Clone, Default)]
pub struct LevelStem;

#[derive(Debug, Clone, Default)]
pub struct LevelDataAndDimensions;

#[derive(Debug, Clone, Default)]
pub struct LevelSettings;

impl LevelSettings {
    pub fn parse(_dynamic: &Dynamic, _config: WorldDataConfiguration) -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct LevelVersion {
    pub level_data_version: i32,
    pub snapshot: bool,
}

impl LevelVersion {
    pub fn parse(dynamic: &Dynamic) -> Self {
        if let Some(ver) = dynamic.get("Version") {
            let level_data_version = match ver.get("Id").map(|d| d.value.clone()) {
                Some(NbtTag::Int(i)) => i,
                _ => 0,
            };
            let snapshot = match ver.get("Snapshot").map(|d| d.value.clone()) {
                Some(NbtTag::Byte(b)) => b != 0,
                _ => false,
            };
            Self {
                level_data_version,
                snapshot,
            }
        } else {
            Self {
                level_data_version: 0,
                snapshot: false,
            }
        }
    }

    pub fn level_data_version(&self) -> i32 {
        self.level_data_version
    }
}

#[derive(Debug, Clone, Default)]
pub struct FeatureFlagSet {
    pub flags: HashSet<String>,
}

pub struct FeatureFlags;

impl FeatureFlags {
    pub fn is_experimental(_set: &FeatureFlagSet) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub enum LevelSummary {
    Normal {
        settings: LevelSettings,
        version: LevelVersion,
        level_id: String,
        requires_manual_conversion: bool,
        locked: bool,
        experimental: bool,
        icon: PathBuf,
    },
    Symlink {
        level_id: String,
        icon: PathBuf,
    },
    Corrupted {
        level_id: String,
        icon: PathBuf,
        last_modified: i64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelResource {
    LevelDataFile,
    OldLevelDataFile,
    IconFile,
    LockFile,
}

impl LevelResource {
    pub fn get_id(&self) -> &str {
        match self {
            Self::LevelDataFile => "level.dat",
            Self::OldLevelDataFile => "level.dat_old",
            Self::IconFile => "icon.png",
            Self::LockFile => "session.lock",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub namespace: String,
    pub path: String,
}

pub struct DimensionType;

impl DimensionType {
    pub fn get_storage_folder(dimension: &ResourceKey, base: &Path) -> PathBuf {
        if dimension.namespace == "minecraft" && dimension.path == "overworld" {
            base.to_path_buf()
        } else {
            base.join("dimensions")
                .join(&dimension.namespace)
                .join(&dimension.path)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RegistryAccess;

pub trait WorldData {
    fn create_tag(&self, registry: &RegistryAccess, player: Option<&NbtTag>) -> NbtTag;
}

#[derive(Debug, Clone)]
pub struct PlayerDataStorage {
    pub fixer: DataFixer,
}

// ---------------------------------------------------------------------------
// NBT helpers (self-contained, uses crate::NbtTag/NbtError)
// ---------------------------------------------------------------------------

fn decode_modified_utf8(buf: &[u8]) -> Result<String, NbtError> {
    let mut out: Vec<u16> = Vec::with_capacity(buf.len());
    let mut i: usize = 0;
    while i < buf.len() {
        let b1 = buf[i];
        match b1 & 0xF0 {
            0x00 | 0x10 | 0x20 | 0x30 | 0x40 | 0x50 | 0x60 | 0x70 => {
                if b1 == 0 {
                    return Err(NbtError::InvalidMutf8);
                }
                out.push(b1 as u16);
                i = i.wrapping_add(1);
            }
            0xC0 | 0xD0 => {
                if i.wrapping_add(1) >= buf.len() {
                    return Err(NbtError::InvalidMutf8);
                }
                let b2 = buf[i.wrapping_add(1)];
                if (b2 & 0xC0) != 0x80 {
                    return Err(NbtError::InvalidMutf8);
                }
                let ch = (((b1 & 0x1F) as u16) << 6) | ((b2 & 0x3F) as u16);
                out.push(ch);
                i = i.wrapping_add(2);
            }
            0xE0 => {
                if i.wrapping_add(2) >= buf.len() {
                    return Err(NbtError::InvalidMutf8);
                }
                let b2 = buf[i.wrapping_add(1)];
                let b3 = buf[i.wrapping_add(2)];
                if (b2 & 0xC0) != 0x80 || (b3 & 0xC0) != 0x80 {
                    return Err(NbtError::InvalidMutf8);
                }
                let ch = (((b1 & 0x0F) as u16) << 12)
                    | (((b2 & 0x3F) as u16) << 6)
                    | ((b3 & 0x3F) as u16);
                out.push(ch);
                i = i.wrapping_add(3);
            }
            _ => return Err(NbtError::InvalidMutf8),
        }
    }
    String::from_utf16(&out).map_err(|_| NbtError::InvalidMutf8)
}

fn encode_modified_utf8(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for ch in s.encode_utf16() {
        let c = ch as u32;
        if c >= 0x0001 && c <= 0x007F {
            out.push(c as u8);
        } else if c == 0 || (c >= 0x0080 && c <= 0x07FF) {
            out.push((0xC0u32 | ((c >> 6) & 0x1F)) as u8);
            out.push((0x80u32 | (c & 0x3F)) as u8);
        } else {
            out.push((0xE0u32 | ((c >> 12) & 0x0F)) as u8);
            out.push((0x80u32 | ((c >> 6) & 0x3F)) as u8);
            out.push((0x80u32 | (c & 0x3F)) as u8);
        }
    }
    out
}

fn read_nbt_string<R: Read>(r: &mut R) -> Result<String, NbtError> {
    let len = r.read_u16::<BigEndian>()? as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    decode_modified_utf8(&buf)
}

fn write_nbt_string<W: Write>(w: &mut W, s: &str) -> io::Result<()> {
    let encoded = encode_modified_utf8(s);
    if encoded.len() > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "String too long",
        ));
    }
    w.write_u16::<BigEndian>(encoded.len() as u16)?;
    w.write_all(&encoded)
}

fn read_nbt_payload<R: Read>(r: &mut R, tag_id: u8, depth: usize) -> Result<NbtTag, NbtError> {
    if depth > 512 {
        return Err(NbtError::DepthExceeded(512));
    }
    match tag_id {
        1 => Ok(NbtTag::Byte(r.read_i8()?)),
        2 => Ok(NbtTag::Short(r.read_i16::<BigEndian>()?)),
        3 => Ok(NbtTag::Int(r.read_i32::<BigEndian>()?)),
        4 => Ok(NbtTag::Long(r.read_i64::<BigEndian>()?)),
        5 => Ok(NbtTag::Float(r.read_f32::<BigEndian>()?)),
        6 => Ok(NbtTag::Double(r.read_f64::<BigEndian>()?)),
        7 => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            let mut buf = vec![0u8; len as usize];
            r.read_exact(&mut buf)?;
            Ok(NbtTag::ByteArray(buf.into_iter().map(|b| b as i8).collect()))
        }
        8 => Ok(NbtTag::String(read_nbt_string(r)?)),
        9 => {
            let elem_type = r.read_u8()?;
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            if elem_type == 0 && len > 0 {
                return Err(NbtError::MissingListType);
            }
            let mut items = Vec::with_capacity(len as usize);
            let mut i: i32 = 0;
            while i < len {
                items.push(read_nbt_payload(r, elem_type, depth.wrapping_add(1))?);
                i = i.wrapping_add(1);
            }
            Ok(NbtTag::List { elem_type, items })
        }
        10 => read_nbt_compound_payload(r, depth.wrapping_add(1)),
        11 => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            let mut items = Vec::with_capacity(len as usize);
            let mut i: i32 = 0;
            while i < len {
                items.push(r.read_i32::<BigEndian>()?);
                i = i.wrapping_add(1);
            }
            Ok(NbtTag::IntArray(items))
        }
        12 => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            let mut items = Vec::with_capacity(len as usize);
            let mut i: i32 = 0;
            while i < len {
                items.push(r.read_i64::<BigEndian>()?);
                i = i.wrapping_add(1);
            }
            Ok(NbtTag::LongArray(items))
        }
        _ => Err(NbtError::UnknownTag(tag_id)),
    }
}

fn read_nbt_compound_payload<R: Read>(r: &mut R, depth: usize) -> Result<NbtTag, NbtError> {
    if depth > 512 {
        return Err(NbtError::DepthExceeded(512));
    }
    let mut entries = Vec::new();
    loop {
        let tag_id = r.read_u8()?;
        if tag_id == 0 {
            break;
        }
        let name = read_nbt_string(r)?;
        let payload = read_nbt_payload(r, tag_id, depth.wrapping_add(1))?;
        entries.push((name, payload));
    }
    Ok(NbtTag::Compound(entries))
}

fn read_nbt_root_compound<R: Read>(r: &mut R) -> Result<NbtTag, NbtError> {
    let tag_id = r.read_u8()?;
    if tag_id != 10 {
        return Err(NbtError::InvalidRoot(tag_id));
    }
    let _name = read_nbt_string(r)?;
    read_nbt_compound_payload(r, 0)
}

fn write_nbt_payload<W: Write>(w: &mut W, tag: &NbtTag) -> io::Result<()> {
    match tag {
        NbtTag::Byte(v) => w.write_i8(*v),
        NbtTag::Short(v) => w.write_i16::<BigEndian>(*v),
        NbtTag::Int(v) => w.write_i32::<BigEndian>(*v),
        NbtTag::Long(v) => w.write_i64::<BigEndian>(*v),
        NbtTag::Float(v) => w.write_f32::<BigEndian>(*v),
        NbtTag::Double(v) => w.write_f64::<BigEndian>(*v),
        NbtTag::ByteArray(v) => {
            w.write_i32::<BigEndian>(v.len() as i32)?;
            for b in v {
                w.write_i8(*b)?;
            }
            Ok(())
        }
        NbtTag::String(s) => write_nbt_string(w, s),
        NbtTag::List { elem_type, items } => {
            w.write_u8(*elem_type)?;
            w.write_i32::<BigEndian>(items.len() as i32)?;
            for item in items {
                write_nbt_payload(w, item)?;
            }
            Ok(())
        }
        NbtTag::Compound(entries) => {
            for (name, val) in entries {
                w.write_u8(val.id())?;
                write_nbt_string(w, name)?;
                write_nbt_payload(w, val)?;
            }
            w.write_u8(0)
        }
        NbtTag::IntArray(v) => {
            w.write_i32::<BigEndian>(v.len() as i32)?;
            for i in v {
                w.write_i32::<BigEndian>(*i)?;
            }
            Ok(())
        }
        NbtTag::LongArray(v) => {
            w.write_i32::<BigEndian>(v.len() as i32)?;
            for i in v {
                w.write_i64::<BigEndian>(*i)?;
            }
            Ok(())
        }
    }
}

fn write_nbt_root<W: Write>(w: &mut W, name: &str, tag: &NbtTag) -> io::Result<()> {
    w.write_u8(tag.id())?;
    write_nbt_string(w, name)?;
    write_nbt_payload(w, tag)
}

fn write_nbt_compressed_file(path: &Path, tag: &NbtTag) -> io::Result<()> {
    let file = File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    write_nbt_root(&mut encoder, "", tag)?;
    encoder.finish()?;
    Ok(())
}

fn set_compound_string(tag: &mut NbtTag, key: &str, value: &str) {
    if let NbtTag::Compound(entries) = tag {
        if let Some((_, v)) = entries.iter_mut().rev().find(|(k, _)| k == key) {
            *v = NbtTag::String(value.to_string());
            return;
        }
        entries.push((key.to_string(), NbtTag::String(value.to_string())));
    }
}

fn remove_compound_key(tag: &mut NbtTag, key: &str) {
    if let NbtTag::Compound(entries) = tag {
        if let Some(pos) = entries.iter().rposition(|(k, _)| k == key) {
            entries.remove(pos);
        }
    }
}

fn create_directories_safe(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

fn find_available_name(dir: &Path, base: &str, ext: &str) -> String {
    let candidate = format!("{}{}", base, ext);
    if !dir.join(&candidate).exists() {
        return candidate;
    }
    let mut i: i32 = 1;
    loop {
        let c = format!("{}_{}{}", base, i, ext);
        if !dir.join(&c).exists() {
            return c;
        }
        i = i.wrapping_add(1);
        if i == 0 {
            return format!("{}_overflow{}", base, ext);
        }
    }
}

fn simple_timestamp() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

fn safe_replace_file(target: &Path, temp: &Path, backup: &Path) -> io::Result<()> {
    if target.exists() {
        let _ = fs::remove_file(backup);
        fs::rename(target, backup)?;
    }
    fs::rename(temp, target)?;
    Ok(())
}

fn safe_replace_or_move_file(target: &Path, source: &Path, backup: &Path, _no_backup: bool) -> bool {
    let res = (|| -> io::Result<()> {
        if target.exists() {
            let _ = fs::remove_file(backup);
            fs::rename(target, backup)?;
        }
        fs::rename(source, target)?;
        Ok(())
    })();
    res.is_ok()
}

fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        let mut k: i32 = 0;
        while k < 8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320u32;
            } else {
                crc >>= 1;
            }
            k = k.wrapping_add(1);
        }
    }
    !crc
}

fn write_zip_stored_entry<W: Write>(
    w: &mut W,
    name: &str,
    data: &[u8],
    central: &mut Vec<u8>,
    offset: &mut u32,
    entry_count: &mut u16,
) -> io::Result<()> {
    let name_bytes = name.as_bytes();
    let crc = crc32_ieee(data);
    let local_offset = *offset;
    let size = data.len() as u32;
    let name_len = name_bytes.len() as u16;

    w.write_all(&0x04034B50u32.to_le_bytes())?;
    w.write_all(&20u16.to_le_bytes())?;
    w.write_all(&0u16.to_le_bytes())?;
    w.write_all(&0u16.to_le_bytes())?;
    w.write_all(&0u16.to_le_bytes())?;
    w.write_all(&0u16.to_le_bytes())?;
    w.write_all(&crc.to_le_bytes())?;
    w.write_all(&size.to_le_bytes())?;
    w.write_all(&size.to_le_bytes())?;
    w.write_all(&name_len.to_le_bytes())?;
    w.write_all(&0u16.to_le_bytes())?;
    w.write_all(name_bytes)?;
    w.write_all(data)?;

    *offset = offset
        .wrapping_add(30u32)
        .wrapping_add(name_bytes.len() as u32)
        .wrapping_add(size);

    central.extend_from_slice(&0x02014B50u32.to_le_bytes());
    central.extend_from_slice(&20u16.to_le_bytes());
    central.extend_from_slice(&20u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&crc.to_le_bytes());
    central.extend_from_slice(&size.to_le_bytes());
    central.extend_from_slice(&size.to_le_bytes());
    central.extend_from_slice(&name_len.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u32.to_le_bytes());
    central.extend_from_slice(&local_offset.to_le_bytes());
    central.extend_from_slice(name_bytes);

    *entry_count = entry_count.wrapping_add(1);
    Ok(())
}

fn walk_zip(
    current: &Path,
    base: &Path,
    level_id_path: &Path,
    w: &mut BufWriter<File>,
    central: &mut Vec<u8>,
    offset: &mut u32,
    entry_count: &mut u16,
) -> io::Result<()> {
    if current.is_dir() {
        let mut children: Vec<PathBuf> = fs::read_dir(current)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        children.sort();
        for p in children {
            walk_zip(&p, base, level_id_path, w, central, offset, entry_count)?;
        }
    } else {
        if current
            .file_name()
            .map(|n| n == "session.lock")
            .unwrap_or(false)
        {
            return Ok(());
        }
        let rel = current.strip_prefix(base).unwrap_or(current);
        let zip_name = level_id_path
            .join(rel)
            .to_string_lossy()
            .replace('\\', "/");
        let data = fs::read(current)?;
        write_zip_stored_entry(w, &zip_name, &data, central, offset, entry_count)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// LevelStorageSource
// ---------------------------------------------------------------------------

pub struct LevelStorageSource {
    pub base_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub fixer_upper: DataFixer,
    pub world_dir_validator: DirectoryValidator,
}

impl LevelStorageSource {
    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#<init>
    pub fn new(
        base_dir: PathBuf,
        backup_dir: PathBuf,
        world_dir_validator: DirectoryValidator,
        fixer_upper: DataFixer,
    ) -> io::Result<Self> {
        fs::create_dir_all(&base_dir)?;
        Ok(Self {
            base_dir,
            backup_dir,
            fixer_upper,
            world_dir_validator,
        })
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#parseValidator
    pub fn parse_validator(path: &Path) -> DirectoryValidator {
        if path.exists() {
            if let Ok(f) = File::open(path) {
                let reader = BufReader::new(f);
                if let Ok(allow) = PathAllowList::read_plain(reader) {
                    return DirectoryValidator::new(allow);
                }
            }
        }
        DirectoryValidator::no_symlinks()
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#createDefault
    pub fn create_default(path: PathBuf) -> io::Result<Self> {
        let validator = Self::parse_validator(&path.join(ALLOWED_SYMLINKS_CONFIG_NAME));
        let backup_dir = path.join("../backups");
        Self::new(path, backup_dir, validator, DataFixer)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#readDataConfig
    pub fn read_data_config(_dynamic: &Dynamic) -> WorldDataConfiguration {
        WorldDataConfiguration
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getPackConfig
    pub fn get_pack_config(
        dynamic: &Dynamic,
        pack_repository: PackRepository,
        safe_mode: bool,
    ) -> PackConfig {
        PackConfig {
            pack_repository,
            data_configuration: Self::read_data_config(dynamic),
            safe_mode,
            stable: false,
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getLevelDataAndDimensions
    pub fn get_level_data_and_dimensions(
        _dynamic: &Dynamic,
        _data_config: WorldDataConfiguration,
        _stems: Registry<LevelStem>,
        _provider: HolderLookupProvider,
    ) -> LevelDataAndDimensions {
        LevelDataAndDimensions
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getName
    pub fn get_name(&self) -> &str {
        "Anvil"
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#findLevelCandidates
    pub fn find_level_candidates(&self) -> Result<LevelCandidates, LevelStorageException> {
        if !self.base_dir.is_dir() {
            return Err(LevelStorageException {
                message: Component::translatable("selectWorld.load_folder_access"),
            });
        }
        let rd = fs::read_dir(&self.base_dir).map_err(|_| LevelStorageException {
            message: Component::translatable("selectWorld.load_folder_access"),
        })?;
        let mut levels = Vec::new();
        for entry in rd {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir = LevelDirectory::new(path);
            if dir.data_file().is_file() || dir.old_data_file().is_file() {
                levels.push(dir);
            }
        }
        Ok(LevelCandidates { levels })
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#loadLevelSummaries
    pub fn load_level_summaries(&self, candidates: LevelCandidates) -> Vec<LevelSummary> {
        let mut results = Vec::with_capacity(candidates.levels.len());
        for dir in &candidates.levels {
            let locked = DirectoryLock::is_locked(&dir.path()).unwrap_or(false);
            if let Some(summary) = self.read_level_summary(dir, locked) {
                results.push(summary);
            }
        }
        results
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getStorageVersion
    fn get_storage_version(&self) -> i32 {
        19133
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#readLevelDataTagRaw
    pub fn read_level_data_tag_raw(path: &Path) -> io::Result<NbtTag> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        read_nbt_root_compound(&mut decoder)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#readLevelDataTagFixed
    pub fn read_level_data_tag_fixed(path: &Path, fixer: &DataFixer) -> io::Result<Dynamic> {
        let tag = Self::read_level_data_tag_raw(path)?;
        let data = match &tag {
            NbtTag::Compound(entries) => entries
                .iter()
                .rev()
                .find(|(k, _)| k == TAG_DATA)
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| NbtTag::Compound(Vec::new())),
            _ => NbtTag::Compound(Vec::new()),
        };
        let version = NbtUtils::get_data_version(&data);
        let mut dynamic =
            DataFixTypes::Level.update_to_current_version(fixer, Dynamic::new(data), version);
        dynamic = dynamic.update("Player", |p| {
            DataFixTypes::Player.update_to_current_version(fixer, p, version)
        });
        dynamic = dynamic.update("WorldGenSettings", |w| {
            DataFixTypes::WorldGenSettings.update_to_current_version(fixer, w, version)
        });
        Ok(dynamic)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#readLevelSummary
    fn read_level_summary(&self, level_dir: &LevelDirectory, locked: bool) -> Option<LevelSummary> {
        let path = level_dir.data_file();
        if path.exists() {
            if fs::symlink_metadata(&path)
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
            {
                let infos = self.world_dir_validator.validate_symlink(&path);
                if !infos.is_empty() {
                    return Some(LevelSummary::Symlink {
                        level_id: level_dir.directory_name(),
                        icon: level_dir.icon_file(),
                    });
                }
            }
            match Self::read_lightweight_data(&path) {
                Ok(Some(NbtTag::Compound(entries))) => {
                    let data = entries
                        .into_iter()
                        .rev()
                        .find(|(k, _)| k == TAG_DATA)
                        .map(|(_, v)| v)
                        .unwrap_or_else(|| NbtTag::Compound(Vec::new()));
                    let version = NbtUtils::get_data_version(&data);
                    let dynamic = DataFixTypes::LevelSummary.update_to_current_version(
                        &self.fixer_upper,
                        Dynamic::new(data),
                        version,
                    );
                    if let Ok(summary) = self.make_level_summary(&dynamic, level_dir, locked) {
                        return Some(summary);
                    }
                }
                Ok(Some(_)) | Ok(None) | Err(_) => {}
            }
        }
        Some(LevelSummary::Corrupted {
            level_id: level_dir.directory_name(),
            icon: level_dir.icon_file(),
            last_modified: get_level_directory_modification_time(level_dir),
        })
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getFileModificationTime
    fn get_level_directory_modification_time_static(dir: &LevelDirectory) -> i64 {
        get_level_directory_modification_time(dir)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#makeLevelSummary
    fn make_level_summary(
        &self,
        dynamic: &Dynamic,
        level_dir: &LevelDirectory,
        locked: bool,
    ) -> Result<LevelSummary, NbtError> {
        let version = LevelVersion::parse(dynamic);
        let ver_num = version.level_data_version();
        if ver_num != 19132 && ver_num != 19133 {
            return Err(NbtError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown data version: {:x}", ver_num as u32),
            )));
        }
        let requires_manual_conversion = ver_num != self.get_storage_version();
        let icon = level_dir.icon_file();
        let data_config = Self::read_data_config(dynamic);
        let settings = LevelSettings::parse(dynamic, data_config);
        let flags = Self::parse_feature_flags_from_summary(dynamic);
        let experimental = FeatureFlags::is_experimental(&flags);
        Ok(LevelSummary::Normal {
            settings,
            version,
            level_id: level_dir.directory_name(),
            requires_manual_conversion,
            locked,
            experimental,
            icon,
        })
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#parseFeatureFlagsFromSummary
    fn parse_feature_flags_from_summary(dynamic: &Dynamic) -> FeatureFlagSet {
        let mut flags = HashSet::new();
        if let Some(enabled) = dynamic.get("enabled_features") {
            for item in enabled.as_stream() {
                if let Some(s) = item.as_string_result() {
                    if !s.is_empty() {
                        flags.insert(s);
                    }
                }
            }
        }
        FeatureFlagSet { flags }
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#readLightweightData
    fn read_lightweight_data(path: &Path) -> io::Result<Option<NbtTag>> {
        match Self::read_level_data_tag_raw(path) {
            Ok(tag) => Ok(Some(tag)),
            Err(e) => Err(e),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#isNewLevelIdAcceptable
    pub fn is_new_level_id_acceptable(&self, name: &str) -> bool {
        let path = self.get_level_path(name);
        match fs::create_dir(&path) {
            Ok(()) => {
                let _ = fs::remove_dir(&path);
                true
            }
            Err(_) => false,
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#levelExists
    pub fn level_exists(&self, name: &str) -> bool {
        self.get_level_path(name).is_dir()
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getLevelPath
    pub fn get_level_path(&self, name: &str) -> PathBuf {
        self.base_dir.join(name)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getBaseDir
    pub fn get_base_dir(&self) -> &Path {
        &self.base_dir
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getBackupPath
    pub fn get_backup_path(&self) -> &Path {
        &self.backup_dir
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#validateAndCreateAccess
    pub fn validate_and_create_access(
        &self,
        name: &str,
    ) -> Result<LevelStorageAccess<'_>, ContentValidationException> {
        let path = self.get_level_path(name);
        let infos = self.world_dir_validator.validate_directory(&path, true);
        if !infos.is_empty() {
            return Err(ContentValidationException { path, infos });
        }
        LevelStorageAccess::new(self, name.to_string(), path.clone()).map_err(|_| {
            ContentValidationException {
                path,
                infos: Vec::new(),
            }
        })
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#createAccess
    pub fn create_access(&self, name: &str) -> io::Result<LevelStorageAccess<'_>> {
        let path = self.get_level_path(name);
        LevelStorageAccess::new(self, name.to_string(), path)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource#getWorldDirValidator
    pub fn get_world_dir_validator(&self) -> &DirectoryValidator {
        &self.world_dir_validator
    }
}

// JAVA: net.minecraft.world.level.storage.LevelStorageSource#getFileModificationTime
pub fn get_file_modification_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn get_level_directory_modification_time(dir: &LevelDirectory) -> i64 {
    let t = get_file_modification_time(&dir.data_file())
        .or_else(|| get_file_modification_time(&dir.old_data_file()));
    match t {
        Some(st) => match st.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => d.as_millis() as i64,
            Err(e) => {
                let ms = e.duration().as_millis() as i64;
                ms.wrapping_neg()
            }
        },
        None => -1i64,
    }
}

// ---------------------------------------------------------------------------
// LevelCandidates
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LevelCandidates {
    pub levels: Vec<LevelDirectory>,
}

impl LevelCandidates {
    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelCandidates#isEmpty
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelCandidates#iterator
    pub fn iterator(&self) -> std::slice::Iter<'_, LevelDirectory> {
        self.levels.iter()
    }
}

// ---------------------------------------------------------------------------
// LevelDirectory
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LevelDirectory {
    pub path: PathBuf,
}

impl LevelDirectory {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelDirectory#directoryName
    pub fn directory_name(&self) -> String {
        self.path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelDirectory#dataFile
    pub fn data_file(&self) -> PathBuf {
        self.resource_path(LevelResource::LevelDataFile)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelDirectory#oldDataFile
    pub fn old_data_file(&self) -> PathBuf {
        self.resource_path(LevelResource::OldLevelDataFile)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelDirectory#corruptedDataFile
    pub fn corrupted_data_file(&self, date_time: &str) -> PathBuf {
        self.path.join(format!(
            "{}_corrupted_{}",
            LevelResource::LevelDataFile.get_id(),
            date_time
        ))
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelDirectory#rawDataFile
    pub fn raw_data_file(&self, date_time: &str) -> PathBuf {
        self.path.join(format!(
            "{}_raw_{}",
            LevelResource::LevelDataFile.get_id(),
            date_time
        ))
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelDirectory#iconFile
    pub fn icon_file(&self) -> PathBuf {
        self.resource_path(LevelResource::IconFile)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelDirectory#lockFile
    pub fn lock_file(&self) -> PathBuf {
        self.resource_path(LevelResource::LockFile)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelDirectory#resourcePath
    pub fn resource_path(&self, resource: LevelResource) -> PathBuf {
        self.path.join(resource.get_id())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

// ---------------------------------------------------------------------------
// LevelStorageAccess
// ---------------------------------------------------------------------------

pub struct LevelStorageAccess<'a> {
    pub lock: DirectoryLock,
    pub level_directory: LevelDirectory,
    level_id: String,
    resources: HashMap<LevelResource, PathBuf>,
    source: &'a LevelStorageSource,
}

impl<'a> LevelStorageAccess<'a> {
    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#<init>
    pub fn new(
        source: &'a LevelStorageSource,
        level_id: String,
        path: PathBuf,
    ) -> io::Result<Self> {
        let level_directory = LevelDirectory::new(path.clone());
        let lock = DirectoryLock::create(&path)?;
        Ok(Self {
            lock,
            level_directory,
            level_id,
            resources: HashMap::new(),
            source,
        })
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#estimateDiskSpace
    pub fn estimate_disk_space(&self) -> i64 {
        i64::MAX
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#checkForLowDiskSpace
    pub fn check_for_low_disk_space(&self) -> bool {
        self.estimate_disk_space() < DISK_SPACE_WARNING_THRESHOLD
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#safeClose
    pub fn safe_close(&mut self) {
        let _ = self.close();
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#parent
    pub fn parent(&self) -> &LevelStorageSource {
        self.source
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getLevelDirectory
    pub fn get_level_directory(&self) -> &LevelDirectory {
        &self.level_directory
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getLevelId
    pub fn get_level_id(&self) -> &str {
        &self.level_id
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getLevelPath
    pub fn get_level_path(&mut self, resource: LevelResource) -> PathBuf {
        if let Some(p) = self.resources.get(&resource) {
            return p.clone();
        }
        let p = self.level_directory.resource_path(resource);
        self.resources.insert(resource, p.clone());
        p
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getDimensionPath
    pub fn get_dimension_path(&self, dimension: &ResourceKey) -> PathBuf {
        DimensionType::get_storage_folder(dimension, &self.level_directory.path)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#checkLock
    fn check_lock(&self) {
        if !self.lock.is_valid() {
            panic!("Lock is no longer valid");
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#createPlayerStorage
    pub fn create_player_storage(&self) -> PlayerDataStorage {
        self.check_lock();
        PlayerDataStorage {
            fixer: self.source.fixer_upper.clone(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getSummary
    pub fn get_summary(&self, dynamic: &Dynamic) -> Result<LevelSummary, NbtError> {
        self.check_lock();
        self.source
            .make_level_summary(dynamic, &self.level_directory, false)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getDataTag
    pub fn get_data_tag(&self) -> io::Result<Dynamic> {
        self.get_data_tag_flag(false)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getDataTagFallback
    pub fn get_data_tag_fallback(&self) -> io::Result<Dynamic> {
        self.get_data_tag_flag(true)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getDataTag
    fn get_data_tag_flag(&self, fallback: bool) -> io::Result<Dynamic> {
        self.check_lock();
        let path = if fallback {
            self.level_directory.old_data_file()
        } else {
            self.level_directory.data_file()
        };
        LevelStorageSource::read_level_data_tag_fixed(&path, &self.source.fixer_upper)
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#saveDataTag
    pub fn save_data_tag(&self, registry: &RegistryAccess, world_data: &dyn WorldData) {
        self.save_data_tag_with_player(registry, world_data, None);
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#saveDataTag
    pub fn save_data_tag_with_player(
        &self,
        registry: &RegistryAccess,
        world_data: &dyn WorldData,
        player: Option<&NbtTag>,
    ) {
        let tag = world_data.create_tag(registry, player);
        let root = NbtTag::Compound(vec![("Data".to_string(), tag)]);
        self.save_level_data(&root);
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#saveLevelData
    fn save_level_data(&self, tag: &NbtTag) {
        let dir = &self.level_directory.path;
        let result = (|| -> io::Result<()> {
            let tmp = dir.join("level.dat.tmp");
            write_nbt_compressed_file(&tmp, tag)?;
            let old = self.level_directory.old_data_file();
            let target = self.level_directory.data_file();
            safe_replace_file(&target, &tmp, &old)?;
            Ok(())
        })();
        let _ = result;
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getIconFile
    pub fn get_icon_file(&self) -> Option<PathBuf> {
        if !self.lock.is_valid() {
            return None;
        }
        Some(self.level_directory.icon_file())
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#deleteLevel
    pub fn delete_level(&mut self) -> io::Result<()> {
        self.check_lock();
        let lock_path = self.level_directory.lock_file();
        let mut last_err: Option<io::Error> = None;
        let mut attempt: i32 = 1;
        while attempt <= 5 {
            let res = delete_level_walk(&self.level_directory.path, &lock_path, &mut self.lock);
            match res {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if attempt >= 5 {
                        return Err(e);
                    }
                    last_err = Some(e);
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
            attempt = attempt.wrapping_add(1);
        }
        Err(last_err.unwrap_or_else(|| io::Error::other("delete failed")))
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#renameLevel
    pub fn rename_level(&self, name: &str) -> io::Result<()> {
        let name = name.trim().to_string();
        self.modify_level_data_without_datafix(|data| {
            set_compound_string(data, "LevelName", &name);
        })
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#renameAndDropPlayer
    pub fn rename_and_drop_player(&self, name: &str) -> io::Result<()> {
        let name = name.trim().to_string();
        self.modify_level_data_without_datafix(|data| {
            set_compound_string(data, "LevelName", &name);
            remove_compound_key(data, "Player");
        })
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#modifyLevelDataWithoutDatafix
    fn modify_level_data_without_datafix<F: FnOnce(&mut NbtTag)>(&self, f: F) -> io::Result<()> {
        self.check_lock();
        let mut tag = LevelStorageSource::read_level_data_tag_raw(&self.level_directory.data_file())?;
        if let NbtTag::Compound(ref mut entries) = tag {
            if let Some((_, data)) = entries.iter_mut().rev().find(|(k, _)| k == TAG_DATA) {
                f(data);
            }
        }
        self.save_level_data(&tag);
        Ok(())
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#makeWorldBackup
    pub fn make_world_backup(&self) -> io::Result<u64> {
        self.check_lock();
        let timestamp = simple_timestamp();
        let base_name = format!("{}_{}", timestamp, self.level_id);
        let backup_path = self.source.get_backup_path().to_path_buf();
        create_directories_safe(&backup_path)?;
        let zip_name = find_available_name(&backup_path, &base_name, ".zip");
        let zip_path = backup_path.join(zip_name);

        let file = File::create(&zip_path)?;
        let mut w = BufWriter::new(file);
        let mut central: Vec<u8> = Vec::new();
        let mut offset: u32 = 0;
        let mut entry_count: u16 = 0;
        let level_id_path = PathBuf::from(&self.level_id);
        let base = self.level_directory.path.clone();

        walk_zip(
            &base,
            &base,
            &level_id_path,
            &mut w,
            &mut central,
            &mut offset,
            &mut entry_count,
        )?;

        let central_offset = offset;
        let central_size = central.len() as u32;
        w.write_all(&central)?;
        w.write_all(&0x06054B50u32.to_le_bytes())?;
        w.write_all(&0u16.to_le_bytes())?;
        w.write_all(&0u16.to_le_bytes())?;
        w.write_all(&entry_count.to_le_bytes())?;
        w.write_all(&entry_count.to_le_bytes())?;
        w.write_all(&central_size.to_le_bytes())?;
        w.write_all(&central_offset.to_le_bytes())?;
        w.write_all(&0u16.to_le_bytes())?;
        w.flush()?;

        fs::metadata(&zip_path).map(|m| m.len())
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#hasWorldData
    pub fn has_world_data(&self) -> bool {
        self.level_directory.data_file().exists() || self.level_directory.old_data_file().exists()
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#close
    pub fn close(&mut self) -> io::Result<()> {
        self.lock.close()
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#restoreLevelDataFromOld
    pub fn restore_level_data_from_old(&self) -> bool {
        let now_str = simple_timestamp();
        safe_replace_or_move_file(
            &self.level_directory.data_file(),
            &self.level_directory.old_data_file(),
            &self.level_directory.corrupted_data_file(&now_str),
            true,
        )
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageSource.LevelStorageAccess#getFileModificationTime
    pub fn get_file_modification_time_flag(&self, fallback: bool) -> Option<SystemTime> {
        let path = if fallback {
            self.level_directory.old_data_file()
        } else {
            self.level_directory.data_file()
        };
        get_file_modification_time(&path)
    }
}

fn delete_level_walk(dir: &Path, lock_path: &Path, lock: &mut DirectoryLock) -> io::Result<()> {
    fn walk(path: &Path, base: &Path, lock_path: &Path, lock: &mut DirectoryLock) -> io::Result<()> {
        if path.is_dir() && !path.is_symlink() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                walk(&entry.path(), base, lock_path, lock)?;
            }
            if path == base {
                lock.close()?;
                let _ = fs::remove_file(lock_path);
            }
            fs::remove_dir(path)?;
        } else if path != lock_path {
            fs::remove_file(path)?;
        }
        Ok(())
    }
    walk(dir, dir, lock_path, lock)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "mc_nbt_lss_{}_{}_{}",
            name,
            std::process::id(),
            simple_timestamp()
        ));
        p
    }

    #[test]
    fn test_storage_version_and_disk_threshold() {
        let dir = temp_dir("ver");
        let _ = fs::remove_dir_all(&dir);
        let src = LevelStorageSource::new(
            dir.clone(),
            dir.join("bak"),
            DirectoryValidator::no_symlinks(),
            DataFixer,
        )
        .unwrap();
        assert_eq!(src.get_storage_version(), 19133);
        assert_eq!(src.get_name(), "Anvil");

        assert!(67_108_863i64 < DISK_SPACE_WARNING_THRESHOLD);
        assert!(!(67_108_864i64 < DISK_SPACE_WARNING_THRESHOLD));
        assert!((-1i64) < DISK_SPACE_WARNING_THRESHOLD);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_level_directory_paths_and_candidates() {
        let dir = LevelDirectory::new(PathBuf::from("/tmp/world"));
        assert_eq!(dir.directory_name(), "world");
        assert_eq!(dir.data_file(), PathBuf::from("/tmp/world/level.dat"));
        assert_eq!(dir.old_data_file(), PathBuf::from("/tmp/world/level.dat_old"));
        assert_eq!(dir.icon_file(), PathBuf::from("/tmp/world/icon.png"));
        assert_eq!(dir.lock_file(), PathBuf::from("/tmp/world/session.lock"));
        assert_eq!(
            dir.corrupted_data_file("2024-01-01"),
            PathBuf::from("/tmp/world/level.dat_corrupted_2024-01-01")
        );
        assert_eq!(
            dir.raw_data_file("2024-01-01"),
            PathBuf::from("/tmp/world/level.dat_raw_2024-01-01")
        );

        let candidates = LevelCandidates {
            levels: vec![dir.clone()],
        };
        assert!(!candidates.is_empty());
        assert_eq!(candidates.iterator().count(), 1);
    }

    #[test]
    fn test_nbt_roundtrip_float_bits_and_negative() {
        let dir = temp_dir("nbt");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("level.dat");

        let nan_f32 = f32::from_bits(0x7FC0_0001);
        let neg_zero_f32 = f32::from_bits(0x8000_0000);
        let nan_f64 = f64::from_bits(0x7FF8_0000_0000_0001);
        let neg_zero_f64 = f64::from_bits(0x8000_0000_0000_0000);

        let tag = NbtTag::Compound(vec![(
            "Data".to_string(),
            NbtTag::Compound(vec![
                ("neg_int".to_string(), NbtTag::Int(-1)),
                ("min_long".to_string(), NbtTag::Long(i64::MIN)),
                ("neg_short".to_string(), NbtTag::Short(i16::MIN)),
                ("neg_byte".to_string(), NbtTag::Byte(i8::MIN)),
                ("nan_f".to_string(), NbtTag::Float(nan_f32)),
                ("neg_zero_f".to_string(), NbtTag::Float(neg_zero_f32)),
                ("nan_d".to_string(), NbtTag::Double(nan_f64)),
                ("neg_zero_d".to_string(), NbtTag::Double(neg_zero_f64)),
            ]),
        )]);

        write_nbt_compressed_file(&path, &tag).unwrap();
        let read_back = LevelStorageSource::read_level_data_tag_raw(&path).unwrap();
        let data = read_back.compound_get("Data").unwrap();

        assert_eq!(data.compound_get("neg_int"), Some(&NbtTag::Int(-1)));
        assert_eq!(data.compound_get("min_long"), Some(&NbtTag::Long(i64::MIN)));
        assert_eq!(
            data.compound_get("neg_short"),
            Some(&NbtTag::Short(i16::MIN))
        );
        assert_eq!(data.compound_get("neg_byte"), Some(&NbtTag::Byte(i8::MIN)));

        match data.compound_get("nan_f").unwrap() {
            NbtTag::Float(v) => assert_eq!(v.to_bits(), 0x7FC0_0001),
            _ => panic!("expected float"),
        }
        match data.compound_get("neg_zero_f").unwrap() {
            NbtTag::Float(v) => assert_eq!(v.to_bits(), 0x8000_0000),
            _ => panic!("expected float"),
        }
        match data.compound_get("nan_d").unwrap() {
            NbtTag::Double(v) => assert_eq!(v.to_bits(), 0x7FF8_0000_0000_0001),
            _ => panic!("expected double"),
        }
        match data.compound_get("neg_zero_d").unwrap() {
            NbtTag::Double(v) => assert_eq!(v.to_bits(), 0x8000_0000_0000_0000),
            _ => panic!("expected double"),
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_modification_time_missing() {
        let missing = std::env::temp_dir().join("mc_nbt_lss_missing_12345_level.dat");
        let _ = fs::remove_file(&missing);
        assert!(get_file_modification_time(&missing).is_none());

        let dir = LevelDirectory::new(std::env::temp_dir().join("mc_nbt_lss_missing_dir_12345"));
        assert_eq!(get_level_directory_modification_time(&dir), -1i64);
    }

    #[test]
    fn test_level_exists_and_acceptability() {
        let base = temp_dir("exists");
        let _ = fs::remove_dir_all(&base);
        let src = LevelStorageSource::new(
            base.clone(),
            base.join("backups"),
            DirectoryValidator::no_symlinks(),
            DataFixer,
        )
        .unwrap();

        assert!(!src.level_exists("nonexistent"));
        assert!(src.is_new_level_id_acceptable("new_world"));
        assert!(!src.level_exists("new_world"));

        fs::create_dir_all(base.join("existing")).unwrap();
        assert!(src.level_exists("existing"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_make_level_summary_rejects_unknown_version_hex() {
        let base = temp_dir("summary");
        let _ = fs::remove_dir_all(&base);
        let src = LevelStorageSource::new(
            base.clone(),
            base.join("bak"),
            DirectoryValidator::no_symlinks(),
            DataFixer,
        )
        .unwrap();
        let level_dir = LevelDirectory::new(base.join("world1"));
        fs::create_dir_all(&level_dir.path).unwrap();

        let dyn_tag = Dynamic::new(NbtTag::Compound(vec![(
            "Version".to_string(),
            NbtTag::Compound(vec![("Id".to_string(), NbtTag::Int(-1))]),
        )]));
        let err = src
            .make_level_summary(&dyn_tag, &level_dir, false)
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("ffffffff"), "expected hex for -1, got: {}", msg);

        let _ = fs::remove_dir_all(&base);
    }
}
