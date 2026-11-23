use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

/// Abstraction over Mojang Codec encode/decode for event log values.
/// Self-contained replacement for `Codec<T>` with `JsonOps.INSTANCE`.
pub trait EventCodec<T> {
    // JAVA: com.mojang.serialization.Codec#encodeStart (adapted)
    fn encode(&self, value: &T) -> io::Result<serde_json::Value>;
    // JAVA: com.mojang.serialization.Codec#parse (adapted)
    fn decode(&self, value: &serde_json::Value) -> io::Result<T>;
}

/// Reader for JSON event log streams.
/// Faithful port of `net.minecraft.util.eventlog.JsonEventLogReader`.
pub struct JsonEventLogReader<T, C: EventCodec<T>> {
    file: Arc<Mutex<File>>,
    codec: Arc<C>,
    position: u64,
    closed: bool,
    reference_count: Option<Arc<AtomicI32>>,
    _marker: PhantomData<fn() -> T>,
}

impl<T, C: EventCodec<T>> JsonEventLogReader<T, C> {
    // JAVA: net.minecraft.util.eventlog.JsonEventLogReader#next
    pub fn next(&mut self) -> io::Result<Option<T>> {
        if self.closed {
            return Ok(None);
        }

        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("event log mutex poisoned"))?;
        file.seek(SeekFrom::Start(self.position))?;

        // Read one JSON value from current position using a buffered reader view.
        // We clone the file handle for buffered reading to preserve position tracking.
        let clone = file.try_clone()?;
        let mut buf_reader = BufReader::new(clone);
        buf_reader.seek(SeekFrom::Start(self.position))?;

        // Skip whitespace to detect EOF.
        loop {
            let buf = buf_reader.fill_buf()?;
            if buf.is_empty() {
                self.position = buf_reader.stream_position()?;
                return Ok(None);
            }
            let ws_len = buf
                .iter()
                .take_while(|b| b.is_ascii_whitespace())
                .count();
            if ws_len == 0 {
                break;
            }
            buf_reader.consume(ws_len);
        }

        let value: serde_json::Value =
            match serde::Deserialize::deserialize(&mut serde_json::Deserializer::from_reader(
                &mut buf_reader,
            )) {
                Ok(v) => v,
                Err(e) => {
                    if e.is_eof() {
                        self.position = buf_reader.stream_position().unwrap_or(self.position);
                        return Ok(None);
                    }
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e));
                }
            };

        self.position = buf_reader.stream_position()?;
        let decoded = self.codec.decode(&value)?;
        Ok(Some(decoded))
    }

    // JAVA: net.minecraft.util.eventlog.JsonEventLogReader#close
    pub fn close(&mut self) -> io::Result<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        if let Some(rc) = self.reference_count.take() {
            release_reference(&rc, &self.file)?;
        }
        Ok(())
    }
}

impl<T, C: EventCodec<T>> Drop for JsonEventLogReader<T, C> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

// JAVA: net.minecraft.util.eventlog.JsonEventLog#releaseReference
fn release_reference(reference_count: &AtomicI32, file: &Arc<Mutex<File>>) -> io::Result<()> {
    if reference_count.fetch_sub(1, Ordering::AcqRel) <= 1 {
        // Java closes the FileChannel when refcount reaches 0.
        // In Rust, File closes on drop; flush to ensure data is persisted.
        let mut f = file
            .lock()
            .map_err(|_| io::Error::other("event log mutex poisoned"))?;
        f.flush()?;
        // Explicitly drop is automatic; setting len/closing handled by OS on drop.
        // We sync to be faithful to "close".
        f.sync_all()?;
    }
    Ok(())
}

/// Append-only JSON event log with reference-counted readers.
/// Faithful port of `net.minecraft.util.eventlog.JsonEventLog`.
pub struct JsonEventLog<T, C: EventCodec<T>> {
    codec: Arc<C>,
    channel: Arc<Mutex<File>>,
    reference_count: Arc<AtomicI32>,
    _marker: PhantomData<fn(T)>,
}

impl<T, C: EventCodec<T>> JsonEventLog<T, C> {
    // JAVA: net.minecraft.util.eventlog.JsonEventLog#<init>
    pub fn new(codec: C, file: File) -> Self {
        Self {
            codec: Arc::new(codec),
            channel: Arc::new(Mutex::new(file)),
            reference_count: Arc::new(AtomicI32::new(1)),
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.util.eventlog.JsonEventLog#open
    pub fn open(codec: C, path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        Ok(Self::new(codec, file))
    }

    // JAVA: net.minecraft.util.eventlog.JsonEventLog#write
    pub fn write(&self, value: &T) -> io::Result<()> {
        let json = self.codec.encode(value)?;
        let mut file = self
            .channel
            .lock()
            .map_err(|_| io::Error::other("event log mutex poisoned"))?;
        let end = file.seek(SeekFrom::End(0))?;
        let _ = end;
        let line = serde_json::to_string(&json)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        file.flush()?;
        Ok(())
    }

    // JAVA: net.minecraft.util.eventlog.JsonEventLog#openReader
    pub fn open_reader(&self) -> io::Result<JsonEventLogReader<T, C>> {
        let current = self.reference_count.load(Ordering::Acquire);
        if current <= 0 {
            return Err(io::Error::other("Event log has already been closed"));
        }
        self.reference_count.fetch_add(1, Ordering::AcqRel);

        Ok(JsonEventLogReader {
            file: Arc::clone(&self.channel),
            codec: Arc::clone(&self.codec),
            position: 0,
            closed: false,
            reference_count: Some(Arc::clone(&self.reference_count)),
            _marker: PhantomData,
        })
    }

    // JAVA: net.minecraft.util.eventlog.JsonEventLog#close
    pub fn close(&self) -> io::Result<()> {
        self.release_reference()
    }

    // JAVA: net.minecraft.util.eventlog.JsonEventLog#releaseReference
    fn release_reference(&self) -> io::Result<()> {
        release_reference(&self.reference_count, &self.channel)
    }
}

impl<T, C: EventCodec<T>> Drop for JsonEventLog<T, C> {
    fn drop(&mut self) {
        let _ = self.release_reference();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Event(i32);

    struct EventCodecImpl;

    impl EventCodec<Event> for EventCodecImpl {
        fn encode(&self, value: &Event) -> io::Result<serde_json::Value> {
            Ok(serde_json::Value::from(value.0))
        }

        fn decode(&self, value: &serde_json::Value) -> io::Result<Event> {
            let v = value
                .as_i64()
                .and_then(|n| i32::try_from(n).ok())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected i32"))?;
            Ok(Event(v))
        }
    }

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let c = COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
        std::env::temp_dir().join(format!(
            "mc_math_json_event_log_{name}_{}_{}_{}.jsonl",
            std::process::id(),
            nanos,
            c
        ))
    }

    #[test]
    fn write_and_read_round_trip() {
        let path = temp_path("round_trip");
        let log = JsonEventLog::open(EventCodecImpl, &path).unwrap();
        log.write(&Event(1)).unwrap();
        log.write(&Event(-2)).unwrap();
        log.write(&Event(i32::MAX)).unwrap();

        let mut reader = log.open_reader().unwrap();
        assert_eq!(reader.next().unwrap(), Some(Event(1)));
        assert_eq!(reader.next().unwrap(), Some(Event(-2)));
        assert_eq!(reader.next().unwrap(), Some(Event(i32::MAX)));
        assert_eq!(reader.next().unwrap(), None);
        reader.close().unwrap();
        log.close().unwrap();
        let _ = fs::remove_file(path);
    }

    #[test]
    fn open_reader_on_closed_log_errors() {
        let path = temp_path("closed");
        let log = JsonEventLog::open(EventCodecImpl, &path).unwrap();
        log.close().unwrap();
        let err = match log.open_reader() {
            Ok(_) => panic!("expected error"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), io::ErrorKind::Other);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn reader_position_tracks_across_next_calls() {
        let path = temp_path("position");
        let log = JsonEventLog::open(EventCodecImpl, &path).unwrap();
        log.write(&Event(10)).unwrap();
        log.write(&Event(20)).unwrap();

        let mut r1 = log.open_reader().unwrap();
        assert_eq!(r1.next().unwrap(), Some(Event(10)));
        // write after reader opened; reader continues from its own position
        log.write(&Event(30)).unwrap();
        assert_eq!(r1.next().unwrap(), Some(Event(20)));
        assert_eq!(r1.next().unwrap(), Some(Event(30)));
        assert_eq!(r1.next().unwrap(), None);
        r1.close().unwrap();
        log.close().unwrap();
        let _ = fs::remove_file(path);
    }

    #[test]
    fn handles_negative_and_min_max_i32() {
        let path = temp_path("bounds");
        let log = JsonEventLog::open(EventCodecImpl, &path).unwrap();
        log.write(&Event(i32::MIN)).unwrap();
        log.write(&Event(-1)).unwrap();
        log.write(&Event(0)).unwrap();
        log.write(&Event(i32::MAX)).unwrap();

        let mut reader = log.open_reader().unwrap();
        assert_eq!(reader.next().unwrap(), Some(Event(i32::MIN)));
        assert_eq!(reader.next().unwrap(), Some(Event(-1)));
        assert_eq!(reader.next().unwrap(), Some(Event(0)));
        assert_eq!(reader.next().unwrap(), Some(Event(i32::MAX)));
        reader.close().unwrap();
        log.close().unwrap();
        let _ = fs::remove_file(path);
    }

    #[test]
    fn multiple_readers_independent_positions() {
        let path = temp_path("multi_reader");
        let log = JsonEventLog::open(EventCodecImpl, &path).unwrap();
        log.write(&Event(7)).unwrap();
        log.write(&Event(8)).unwrap();

        let mut r1 = log.open_reader().unwrap();
        let mut r2 = log.open_reader().unwrap();
        assert_eq!(r1.next().unwrap(), Some(Event(7)));
        assert_eq!(r2.next().unwrap(), Some(Event(7)));
        assert_eq!(r1.next().unwrap(), Some(Event(8)));
        assert_eq!(r2.next().unwrap(), Some(Event(8)));
        assert_eq!(r1.next().unwrap(), None);
        assert_eq!(r2.next().unwrap(), None);
        r1.close().unwrap();
        r2.close().unwrap();
        log.close().unwrap();
        let _ = fs::remove_file(path);
    }
}
