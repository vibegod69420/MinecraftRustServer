use std::io::{self, Read};

pub struct FastBufferedInputStream<R: Read> {
    inner: R,
    buffer: Vec<u8>,
    limit: usize,
    position: usize,
}

impl<R: Read> FastBufferedInputStream<R> {
    const DEFAULT_BUFFER_SIZE: usize = 8192;

    // JAVA: net.minecraft.util.FastBufferedInputStream#<init>(InputStream)
    pub fn new(inner: R) -> Self {
        Self::with_capacity(inner, Self::DEFAULT_BUFFER_SIZE)
    }

    // JAVA: net.minecraft.util.FastBufferedInputStream#<init>(InputStream, int)
    pub fn with_capacity(inner: R, size: usize) -> Self {
        Self {
            inner,
            buffer: vec![0u8; size],
            limit: 0,
            position: 0,
        }
    }

    // JAVA: net.minecraft.util.FastBufferedInputStream#bytesInBuffer
    fn bytes_in_buffer(&self) -> usize {
        self.limit.wrapping_sub(self.position)
    }

    // JAVA: net.minecraft.util.FastBufferedInputStream#fill
    fn fill(&mut self) -> io::Result<()> {
        self.limit = 0;
        self.position = 0;
        let n = self.inner.read(&mut self.buffer)?;
        if n > 0 {
            self.limit = n;
        }
        Ok(())
    }

    // JAVA: net.minecraft.util.FastBufferedInputStream#read()
    pub fn read_byte(&mut self) -> io::Result<i32> {
        if self.position >= self.limit {
            self.fill()?;
            if self.position >= self.limit {
                return Ok(-1);
            }
        }
        let b = self.buffer[self.position];
        self.position = self.position.wrapping_add(1);
        Ok(b as i32)
    }

    // JAVA: net.minecraft.util.FastBufferedInputStream#read(byte[], int, int)
    pub fn read_into(&mut self, buf: &mut [u8], off: usize, len: usize) -> io::Result<i32> {
        let mut available = self.bytes_in_buffer();
        if available == 0 {
            if len >= self.buffer.len() {
                let n = self.inner.read(&mut buf[off..off.wrapping_add(len)])?;
                return Ok(if n == 0 { -1 } else { n as i32 });
            }
            self.fill()?;
            available = self.bytes_in_buffer();
            if available == 0 {
                return Ok(-1);
            }
        }

        let mut n = len;
        if n > available {
            n = available;
        }
        let src_start = self.position;
        let src_end = src_start.wrapping_add(n);
        let dst_start = off;
        let dst_end = dst_start.wrapping_add(n);
        buf[dst_start..dst_end].copy_from_slice(&self.buffer[src_start..src_end]);
        self.position = self.position.wrapping_add(n);
        Ok(n as i32)
    }

    // JAVA: net.minecraft.util.FastBufferedInputStream#skip
    pub fn skip(&mut self, n: i64) -> io::Result<i64> {
        if n <= 0 {
            return Ok(0);
        }
        let buffered = self.bytes_in_buffer() as i64;
        if buffered <= 0 {
            return self.skip_inner(n);
        }
        let mut to_skip = n;
        if to_skip > buffered {
            to_skip = buffered;
        }
        self.position = (self.position as i64 + to_skip) as usize;
        Ok(to_skip)
    }

    // JAVA: net.minecraft.util.FastBufferedInputStream#available
    pub fn available(&self) -> io::Result<i32> {
        let buffered = self.bytes_in_buffer() as i32;
        let inner_avail = self.available_inner()?;
        Ok(buffered.wrapping_add(inner_avail))
    }

    // JAVA: net.minecraft.util.FastBufferedInputStream#close
    pub fn close(self) -> io::Result<()> {
        self.close_inner()
    }

    fn skip_inner(&mut self, n: i64) -> io::Result<i64> {
        // Default InputStream.skip behavior approximation for generic Read:
        // read and discard in chunks.
        let mut remaining = n;
        let mut skipped: i64 = 0;
        let mut tmp = [0u8; 8192];
        while remaining > 0 {
            let chunk = std::cmp::min(remaining, tmp.len() as i64) as usize;
            match self.inner.read(&mut tmp[..chunk]) {
                Ok(0) => break,
                Ok(read) => {
                    let r = read as i64;
                    skipped += r;
                    remaining -= r;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(skipped)
    }

    fn available_inner(&self) -> io::Result<i32> {
        // Generic Read has no available(); Java InputStream.available() returns 0 by default.
        Ok(0)
    }

    fn close_inner(self) -> io::Result<()> {
        // Generic Read has no close(); Java InputStream.close() is no-op by default.
        drop(self.inner);
        Ok(())
    }
}

impl<R: Read> Read for FastBufferedInputStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.read_into(buf, 0, buf.len())?;
        if n < 0 {
            Ok(0)
        } else {
            Ok(n as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FastBufferedInputStream;
    use std::io::{self, Cursor, Read};

    #[test]
    fn read_byte_and_eof() {
        let mut s = FastBufferedInputStream::with_capacity(Cursor::new(vec![1u8, 2, 255]), 2);
        assert_eq!(s.read_byte().unwrap(), 1);
        assert_eq!(s.read_byte().unwrap(), 2);
        assert_eq!(s.read_byte().unwrap(), 255);
        assert_eq!(s.read_byte().unwrap(), -1);
    }

    #[test]
    fn read_into_buffer_across_fills() {
        let data: Vec<u8> = (0u8..10).collect();
        let mut s = FastBufferedInputStream::with_capacity(Cursor::new(data), 4);
        let mut out = [0u8; 10];
        let n1 = s.read_into(&mut out, 0, 3).unwrap();
        assert_eq!(n1, 3);
        let n2 = s.read_into(&mut out, 3, 7).unwrap();
        assert!(n2 > 0);
        let mut total = (n1 as usize) + (n2 as usize);
        while total < 10 {
            let n = s.read_into(&mut out, total, 10 - total).unwrap();
            if n < 0 {
                break;
            }
            total += n as usize;
        }
        assert_eq!(total, 10);
        assert_eq!(&out[..], &(0u8..10).collect::<Vec<_>>()[..]);
    }

    #[test]
    fn read_large_bypasses_buffer() {
        let data: Vec<u8> = (0u8..20).collect();
        let mut s = FastBufferedInputStream::with_capacity(Cursor::new(data.clone()), 4);
        let mut out = [0u8; 20];
        let n = s.read_into(&mut out, 0, 20).unwrap();
        assert_eq!(n, 20);
        assert_eq!(&out[..], &data[..]);
    }

    #[test]
    fn skip_within_and_past_buffer() {
        let data: Vec<u8> = (0u8..16).collect();
        let mut s = FastBufferedInputStream::with_capacity(Cursor::new(data), 4);
        assert_eq!(s.read_byte().unwrap(), 0);
        let skipped = s.skip(2).unwrap();
        assert_eq!(skipped, 2);
        assert_eq!(s.read_byte().unwrap(), 3);
        let skipped2 = s.skip(10).unwrap();
        assert!(skipped2 >= 0);
    }

    #[test]
    fn available_and_read_trait() {
        let mut s = FastBufferedInputStream::with_capacity(Cursor::new(vec![10u8, 20, 30]), 8);
        let mut buf = [0u8; 2];
        let n = Read::read(&mut s, &mut buf).unwrap();
        assert_eq!(n, 2);
        assert_eq!(buf, [10, 20]);
        let avail = s.available().unwrap();
        assert!(avail >= 0);
    }

    #[test]
    fn skip_non_positive_returns_zero() {
        let mut s = FastBufferedInputStream::new(Cursor::new(vec![1u8, 2, 3]));
        assert_eq!(s.skip(0).unwrap(), 0);
        assert_eq!(s.skip(-5).unwrap(), 0);
        assert_eq!(s.read_byte().unwrap(), 1);
    }
}
