use std::io::{self, BufRead, BufReader, Read};
use std::marker::PhantomData;

/// Abstraction over Mojang Codec decode for a single event type.
/// Self-contained replacement for `Codec<T>.parse(JsonOps.INSTANCE, ...)`.
pub trait EventCodec<T> {
    // JAVA: com.mojang.serialization.Codec#parse (adapted)
    fn decode(&self, value: &serde_json::Value) -> io::Result<T>;
}

/// Reader for a stream of JSON-encoded events, one value at a time.
/// Faithful port of `net.minecraft.util.eventlog.JsonEventLogReader`.
pub struct JsonEventLogReader<T, C: EventCodec<T>> {
    reader: BufReader<Box<dyn Read>>,
    codec: C,
    closed: bool,
    _marker: PhantomData<fn() -> T>,
}

impl<T, C: EventCodec<T>> JsonEventLogReader<T, C> {
    // JAVA: net.minecraft.util.eventlog.JsonEventLogReader#create
    pub fn create(codec: C, reader: Box<dyn Read>) -> Self {
        Self {
            reader: BufReader::new(reader),
            codec,
            closed: false,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.util.eventlog.JsonEventLogReader#next
    pub fn next(&mut self) -> io::Result<Option<T>> {
        if self.closed {
            return Ok(None);
        }

        // Skip leading whitespace to detect clean EOF like JsonReader.hasNext().
        loop {
            let buf = self.reader.fill_buf()?;
            if buf.is_empty() {
                return Ok(None);
            }
            let ws_len = buf
                .iter()
                .take_while(|b| b.is_ascii_whitespace())
                .count();
            if ws_len == 0 {
                break;
            }
            self.reader.consume(ws_len);
        }

        let value: serde_json::Value =
            match serde::Deserialize::deserialize(&mut serde_json::Deserializer::from_reader(
                &mut self.reader,
            )) {
                Ok(v) => v,
                Err(e) => {
                    if e.is_eof() {
                        return Ok(None);
                    }
                    // JsonParseException -> IOException
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e));
                }
            };

        let decoded = self.codec.decode(&value)?;
        Ok(Some(decoded))
    }

    // JAVA: net.minecraft.util.eventlog.JsonEventLogReader#close
    pub fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        Ok(())
    }
}

impl<T, C: EventCodec<T>> Drop for JsonEventLogReader<T, C> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct I32Codec;

    impl EventCodec<i32> for I32Codec {
        fn decode(&self, value: &serde_json::Value) -> io::Result<i32> {
            value
                .as_i64()
                .and_then(|v| i32::try_from(v).ok())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected i32"))
        }
    }

    struct StringCodec;

    impl EventCodec<String> for StringCodec {
        fn decode(&self, value: &serde_json::Value) -> io::Result<String> {
            value
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected string"))
        }
    }

    #[test]
    fn reads_multiple_json_values() {
        let input = b"1\n2\n3\n";
        let mut reader = JsonEventLogReader::create(I32Codec, Box::new(Cursor::new(input)));
        assert_eq!(reader.next().unwrap(), Some(1));
        assert_eq!(reader.next().unwrap(), Some(2));
        assert_eq!(reader.next().unwrap(), Some(3));
        assert_eq!(reader.next().unwrap(), None);
    }

    #[test]
    fn returns_none_on_empty_input() {
        let mut reader = JsonEventLogReader::create(I32Codec, Box::new(Cursor::new(b"")));
        assert_eq!(reader.next().unwrap(), None);
        assert_eq!(reader.next().unwrap(), None);
    }

    #[test]
    fn handles_negative_and_boundary_integers() {
        let input = b"-1\n-2147483648\n2147483647\n";
        let mut reader = JsonEventLogReader::create(I32Codec, Box::new(Cursor::new(input)));
        assert_eq!(reader.next().unwrap(), Some(-1));
        assert_eq!(reader.next().unwrap(), Some(i32::MIN));
        assert_eq!(reader.next().unwrap(), Some(i32::MAX));
        assert_eq!(reader.next().unwrap(), None);
    }

    #[test]
    fn invalid_json_returns_io_error() {
        let mut reader =
            JsonEventLogReader::create(I32Codec, Box::new(Cursor::new(b"{bad json}\n")));
        let err = reader.next().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn decode_error_is_propagated() {
        let mut reader =
            JsonEventLogReader::create(I32Codec, Box::new(Cursor::new(b"\"not-an-int\"\n")));
        let err = reader.next().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn close_makes_next_return_none() {
        let mut reader = JsonEventLogReader::create(StringCodec, Box::new(Cursor::new(b"\"a\"\n\"b\"\n")));
        assert_eq!(reader.next().unwrap(), Some("a".to_string()));
        reader.close().unwrap();
        assert_eq!(reader.next().unwrap(), None);
    }
}
