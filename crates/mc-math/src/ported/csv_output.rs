use std::fmt;
use std::io::{self, Write};

/// Faithful port of `net.minecraft.util.CsvOutput`.
pub struct CsvOutput<W: Write> {
    output: W,
    column_count: usize,
}

// JAVA: net.minecraft.util.CsvOutput#builder
pub fn builder() -> CsvOutputBuilder {
    CsvOutputBuilder::new()
}

impl<W: Write> CsvOutput<W> {
    // JAVA: net.minecraft.util.CsvOutput#<init>
    fn new(mut output: W, headers: &[String]) -> io::Result<Self> {
        let column_count = headers.len();
        let header_cells: Vec<Option<String>> =
            headers.iter().map(|s| Some(s.clone())).collect();
        let mut out = Self {
            output,
            column_count,
        };
        out.write_line_cells(&header_cells)?;
        Ok(out)
    }

    // JAVA: net.minecraft.util.CsvOutput#writeRow
    pub fn write_row(&mut self, values: &[Option<&dyn fmt::Display>]) -> io::Result<()> {
        if values.len() != self.column_count {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Invalid number of columns, expected {}, but got {}",
                    self.column_count,
                    values.len()
                ),
            ));
        }
        self.write_line(values)
    }

    // JAVA: net.minecraft.util.CsvOutput#writeLine
    fn write_line(&mut self, values: &[Option<&dyn fmt::Display>]) -> io::Result<()> {
        let line = values
            .iter()
            .map(|v| get_string_value(*v))
            .collect::<Vec<_>>()
            .join(",");
        self.output.write_all(format!("{}\r\n", line).as_bytes())
    }

    fn write_line_cells(&mut self, values: &[Option<String>]) -> io::Result<()> {
        let line = values
            .iter()
            .map(|v| match v {
                Some(s) => escape_csv(s),
                None => escape_csv("[null]"),
            })
            .collect::<Vec<_>>()
            .join(",");
        self.output.write_all(format!("{}\r\n", line).as_bytes())
    }
}

// JAVA: net.minecraft.util.CsvOutput#getStringValue
fn get_string_value(value: Option<&dyn fmt::Display>) -> String {
    let s = match value {
        Some(v) => v.to_string(),
        None => "[null]".to_string(),
    };
    escape_csv(&s)
}

fn escape_csv(s: &str) -> String {
    let needs_quote = s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r');
    if !needs_quote {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len().saturating_add(2));
    out.push('"');
    for c in s.chars() {
        if c == '"' {
            out.push('"');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// Faithful port of `net.minecraft.util.CsvOutput$Builder`.
pub struct CsvOutputBuilder {
    headers: Vec<String>,
}

impl CsvOutputBuilder {
    // JAVA: net.minecraft.util.CsvOutput$Builder#<init>
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    // JAVA: net.minecraft.util.CsvOutput$Builder#addColumn
    pub fn add_column(mut self, header: impl Into<String>) -> Self {
        self.headers.push(header.into());
        self
    }

    // JAVA: net.minecraft.util.CsvOutput$Builder#build
    pub fn build<W: Write>(self, writer: W) -> io::Result<CsvOutput<W>> {
        CsvOutput::new(writer, &self.headers)
    }
}

impl Default for CsvOutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::builder;
    use std::fmt;

    fn as_display<T: fmt::Display>(v: &T) -> &dyn fmt::Display {
        v
    }

    #[test]
    fn writes_header_and_rows() {
        let mut buf: Vec<u8> = Vec::new();
        let mut csv = builder().add_column("a").add_column("b").build(&mut buf).unwrap();
        let one: i32 = 1;
        let two: &str = "x";
        csv.write_row(&[Some(as_display(&one)), Some(as_display(&two))]).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(out, "a,b\r\n1,x\r\n");
    }

    #[test]
    fn writes_null_and_escapes_csv() {
        let mut buf: Vec<u8> = Vec::new();
        let mut csv = builder().add_column("v").build(&mut buf).unwrap();
        let comma = "a,b";
        let quote = "a\"b";
        let nl = "a\nb";
        csv.write_row(&[None]).unwrap();
        csv.write_row(&[Some(as_display(&comma))]).unwrap();
        csv.write_row(&[Some(as_display(&quote))]).unwrap();
        csv.write_row(&[Some(as_display(&nl))]).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(out, "v\r\n[null]\r\n\"a,b\"\r\n\"a\"\"b\"\r\n\"a\nb\"\r\n");
    }

    #[test]
    fn column_mismatch_returns_error() {
        let mut buf: Vec<u8> = Vec::new();
        let mut csv = builder().add_column("a").add_column("b").build(&mut buf).unwrap();
        let one: i32 = 1;
        let err = csv.write_row(&[Some(as_display(&one))]).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("expected 2, but got 1"));
    }

    #[test]
    fn empty_builder_writes_empty_header_line() {
        let mut buf: Vec<u8> = Vec::new();
        let mut csv = builder().build(&mut buf).unwrap();
        csv.write_row(&[]).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(out, "\r\n\r\n");
    }
}
