//! Definition of various mergers.
#![allow(unreachable_patterns)]

use crate::error::{ErrorKind, Result};
use crate::util;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

use byteseeker::ByteSeeker;

/// A Merger that can merge multiple sources that implement [`Read`] and [`Seek`] into one.
///
/// Generally speaking, when using `RsMerger`, you'll first call [`new`] to create a
/// merger builder, then call configuration methods to set each option. And eventually, call
/// [`merge_sources_into`] to actually merge multiple sources into a given writer.
///
/// # Behaviours
///
/// The current algorithm to merge sources is described as following:
///
/// 1. If any leading padding is given by [`pad_with`], writes it into the writer.
/// 2. For each source, if skip range is given by [`skip_head`] and
///    [`skip_tail`], writes the contents remaining into the writer.
/// 3. For each source, if forcing ending newline option is set by
///    [`force_ending_newline`], writes a ending newline into the writer if this source
///    is not ends with a newline.
/// 4. For each source, if any inner padding is given by [`pad_with`], writes it into the
///    writer.
/// 5. If any ending padding is given by [`pad_with`], writes it into the writer.
///
/// # Examples
///
/// ```
/// use admerge::{RsMerger, Skip, Pad, Newline, Result};
/// use std::io::Cursor;
///
/// fn main() -> Result<()> {
///     // Cursor implements `Read` and `Seek`.
///     let mut c1 = Cursor::new(" record 1\n date created: 2000/01/01\n");
///     let mut c2 = Cursor::new(" record 2\n date created: 2000/01/01\n");
///     let mut c3 = Cursor::new(" record 3\n date created: 2000/01/01\n");
///     let mut buf = Vec::new();
///
///     // Configures merger.
///     let mut merger = RsMerger::new();
///     merger.skip_tail(Skip::LinesOnce(1));
///     merger.pad_with(Pad::Before(b"header\n"));
///     merger.force_ending_newline(Newline::Lf);
///
///     // Merges sources into one.
///     merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf)?;
///     assert_eq!(
///         std::str::from_utf8(&buf).unwrap(),
///         "header\n record 1\n record 2\n record 3\n date created: 2000/01/01\n"
///     );
///
///     Ok(())
/// }
/// ```
///
/// [`Read`]: std::io::Read
/// [`Seek`]: std::io::Seek
/// [`new`]: RsMerger::new
/// [`pad_with`]: RsMerger::pad_with
/// [`skip_head`]: RsMerger::skip_head
/// [`skip_tail`]: RsMerger::skip_tail
/// [`force_ending_newline`]: RsMerger::force_ending_newline
/// [`merge_sources_into`]: RsMerger::merge_sources_into
#[derive(Debug, Clone)]
pub struct RsMerger<'a> {
    opts: RsMergerOptions<'a>,
}

#[derive(Clone, Debug, Default)]
struct RsMergerOptions<'a> {
    skip_head: Option<Skip<'a>>,
    skip_tail: Option<Skip<'a>>,
    padding: Option<Pad<'a>>,
    newline: Option<Newline>,
}

/// Controls the skip behaviour when merging sources.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Skip<'a> {
    /// Skip a number of bytes from each part.
    Bytes(usize),
    /// Keep the contents of the first part untouched (or the last part if passed by `skip_tail`),
    /// but skip a given number of bytes from the rest parts.
    BytesOnce(usize),
    /// Skip a number of lines from each part.
    Lines(usize),
    /// Keep the contents of the first part untouched (or the last part if passed by `skip_tail`),
    /// but skip a given number of lines from the rest parts.
    LinesOnce(usize),
    /// Skip every byte sequence that matches a given byte pattern from each part.
    /// The given byte pattern must match the first few bytes.
    Repeats(&'a [u8]),
    /// Skip a sequence of bytes until reaching a given byte pattern from each part.
    /// The given byte pattern will be skipped.
    Until(&'a [u8]),
    /// Skip a sequence of bytes until reaching a given byte pattern from each part.
    /// The given byte pattern will not be skipped.
    Before(&'a [u8]),
}

/// Configures where padding will be filled when merging sources.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Pad<'a> {
    /// Fills the given padding before the first source.
    Before(&'a [u8]),
    /// Fills the given padding between two sources.
    Between(&'a [u8]),
    /// Fills the given padding after the last source.
    After(&'a [u8]),
    /// Fills the given paddings before the first source,
    /// between sources and after the last source.
    ///
    /// The argument order is (Before, Between, After).
    Custom(Option<&'a [u8]>, Option<&'a [u8]>, Option<&'a [u8]>),
}

/// The style of a newline, either unix-style `LF` or dos-style `CRLF`.
#[derive(Debug, Clone, Copy)]
pub enum Newline {
    Lf,
    Crlf,
}

impl Default for Newline {
    fn default() -> Self {
        Newline::Lf
    }
}

impl<'a> Default for RsMerger<'a> {
    fn default() -> Self {
        let opts = RsMergerOptions {
            skip_head: None,
            skip_tail: None,
            padding: None,
            newline: None,
        };
        RsMerger { opts }
    }
}

// Public APIs
impl<'a> RsMerger<'a> {
    /// Creates a new `RsMerger` builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use admerge::{RsMerger, Result};
    /// use std::io::Cursor;
    ///
    /// fn main() -> Result<()> {
    ///     let c1 = Cursor::new("foo ");
    ///     let c2 = Cursor::new("bar ");
    ///     let c3 = Cursor::new("baz ");
    ///     let mut buf = Vec::new();
    ///
    ///     let merger = RsMerger::new();
    ///     merger.merge_sources_into(vec![c1, c2, c3], &mut buf)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Configures this merger to skip partial of contents from the head of each source.
    ///
    /// # Examples
    ///
    /// Keeps the first given source untouched, but skips first line from the rest sources.
    ///
    /// ```
    /// use admerge::{RsMerger, Skip, Result};
    /// use std::io::Cursor;
    ///
    /// fn main() -> Result<()> {
    ///     // Cursor implements `Read` and `Seek`.
    ///     let mut c1 = Cursor::new("header\n record 1\n");
    ///     let mut c2 = Cursor::new("header\n record 2\n");
    ///     let mut c3 = Cursor::new("header\n record 3\n");
    ///     let mut buf = Vec::new();
    ///
    ///     // Configures merger.
    ///     let mut merger = RsMerger::new();
    ///     merger.skip_head(Skip::LinesOnce(1));
    ///
    ///     // Merges sources into one.
    ///     merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf)?;
    ///     assert_eq!(
    ///         std::str::from_utf8(&buf).unwrap(),
    ///         "header\n record 1\n record 2\n record 3\n"
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn skip_head(&mut self, skip: Skip<'a>) -> &mut Self {
        self.opts.skip_head = Some(skip);
        self
    }

    /// Configures this merger to skip partial of contents from the tail of each source.
    ///
    /// # Examples
    ///
    /// Keeps the last given source untouched, but skips last line from the rest sources.
    ///
    /// ```
    /// use admerge::{RsMerger, Skip, Result};
    /// use std::io::Cursor;
    ///
    /// fn main() -> Result<()> {
    ///     // Cursor implements `Read` and `Seek`.
    ///     let mut c1 = Cursor::new(" record 1\n date created: 2000/01/01\n");
    ///     let mut c2 = Cursor::new(" record 2\n date created: 2000/01/01\n");
    ///     let mut c3 = Cursor::new(" record 3\n date created: 2000/01/01\n");
    ///     let mut buf = Vec::new();
    ///
    ///     // Configures merger.
    ///     let mut merger = RsMerger::new();
    ///     merger.skip_tail(Skip::LinesOnce(1));
    ///
    ///     // Merges sources into one.
    ///     merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf)?;
    ///     assert_eq!(
    ///         std::str::from_utf8(&buf).unwrap(),
    ///         " record 1\n record 2\n record 3\n date created: 2000/01/01\n"
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn skip_tail(&mut self, skip: Skip<'a>) -> &mut Self {
        self.opts.skip_tail = Some(skip);
        self
    }

    /// Configures this merger to fill some padding before, between or after the given sources.
    ///
    /// # Examples
    ///
    /// ```
    /// use admerge::{RsMerger, Pad, Result};
    /// use std::io::Cursor;
    ///
    /// fn main() -> Result<()> {
    ///     // Cursor implements `Read` and `Seek`.
    ///     let mut c1 = Cursor::new(" record 1 ");
    ///     let mut c2 = Cursor::new(" record 2 ");
    ///     let mut c3 = Cursor::new(" record 3 ");
    ///     let mut buf = Vec::new();
    ///
    ///     // Configures merger.
    ///     let mut merger = RsMerger::new();
    ///     merger.pad_with(Pad::Custom(
    ///         Some(b"header"),
    ///         Some(b"padding"),
    ///         Some(b"date created: 2000/01/01")
    ///     ));
    ///
    ///     // Merges sources into one.
    ///     merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf)?;
    ///     assert_eq!(
    ///         std::str::from_utf8(&buf).unwrap(),
    ///         "header record 1 padding record 2 padding record 3 date created: 2000/01/01"
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn pad_with(&mut self, padding: Pad<'a>) -> &mut Self {
        self.opts.padding = Some(padding);
        self
    }

    /// Configures this merger to force the presence of ending newline after each source.
    ///
    /// Noting that ending newlines are given after sources, not after paddings.
    ///
    /// # Examples
    ///
    /// ```
    /// use admerge::{RsMerger, Newline, Result};
    /// use std::io::Cursor;
    ///
    /// fn main() -> Result<()> {
    ///     // Cursor implements `Read` and `Seek`.
    ///     let mut c1 = Cursor::new(" line 1 ");
    ///     let mut c2 = Cursor::new(" line 2 ");
    ///     let mut c3 = Cursor::new(" line 3 ");
    ///     let mut buf = Vec::new();
    ///
    ///     // Configures merger.
    ///     let mut merger = RsMerger::new();
    ///     merger.force_ending_newline(Newline::Crlf);
    ///
    ///     // Merges sources into one.
    ///     merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf)?;
    ///     assert_eq!(
    ///         std::str::from_utf8(&buf).unwrap(),
    ///         " line 1 \r\n line 2 \r\n line 3 \r\n"
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn force_ending_newline(&mut self, newline: Newline) -> &mut Self {
        self.opts.newline = Some(newline);
        self
    }

    /// Merges the given sources into the given writer according to the given configurations.
    ///
    /// # Errors
    ///
    /// Returns an error variant of [`ErrorKind::NothingPassed`] if the given source vector is
    /// empty;
    ///
    /// Returns an error variant of [`ErrorKind::InvalidSkip`] if the given [`Skip`]s cannot
    /// applied to the given sources;
    ///
    /// Returns an error variant of [`ErrorKind::Io`] if any I/O errors were encountered.
    ///
    /// # Examples
    ///
    /// ```
    /// use admerge::{RsMerger, Skip, Pad, Newline, Result};
    /// use std::io::Cursor;
    ///
    /// fn main() -> Result<()> {
    ///     // Cursor implements `Read` and `Seek`.
    ///     let mut c1 = Cursor::new(" record 1\n date created: 2000/01/01\n");
    ///     let mut c2 = Cursor::new(" record 2\n date created: 2000/01/01\n");
    ///     let mut c3 = Cursor::new(" record 3\n date created: 2000/01/01\n");
    ///     let mut buf = Vec::new();
    ///
    ///     // Configures merger.
    ///     let mut merger = RsMerger::new();
    ///     merger.skip_tail(Skip::LinesOnce(1));
    ///     merger.pad_with(Pad::Before(b"header\n"));
    ///     merger.force_ending_newline(Newline::Lf);
    ///
    ///     // Merges sources into one.
    ///     merger.merge_sources_into(vec![&mut c1, &mut c2, &mut c3], &mut buf)?;
    ///     assert_eq!(
    ///         std::str::from_utf8(&buf).unwrap(),
    ///         "header\n record 1\n record 2\n record 3\n date created: 2000/01/01\n"
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn merge_sources_into<RS, W>(&self, mut sources: Vec<RS>, writer: &mut W) -> Result<()>
    where
        RS: Read + Seek,
        W: Write,
    {
        let len = sources.len();
        if len == 0 {
            return Err(ErrorKind::NothingPassed);
        }

        // Merge first part.
        self.write_contents(&mut sources[0], writer, PartPos::Start)?;
        // Merge inner parts.
        for i in 1..(len - 1) {
            self.write_contents(&mut sources[i], writer, PartPos::Inside)?;
        }
        // Merge last part.
        if len > 1 {
            self.write_contents(&mut sources[len - 1], writer, PartPos::End)?;
        }

        return Ok(());
    }
}

// Indicates the relative position.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PartPos {
    Start,
    Inside,
    End,
}

// Private methods
impl<'a> RsMerger<'a> {
    // Writes the contents (entire or partial) of one part into the writer.
    fn write_contents<RS, W>(&self, reader: &mut RS, writer: &mut W, pos: PartPos) -> Result<()>
    where
        RS: Read + Seek,
        W: Write,
    {
        // Writes padding before this source.
        self.write_padding_before(writer, pos)?;

        // Needs to know if the reader stream ends with a newline or not.
        let endn = util::endswith_newline(reader)?;

        // Gets the stream length of the given reader;
        let stream_len = util::seek_to_end(reader)? as usize;

        // Resets the cursor first.
        util::seek_to_start(reader)?;

        if !self.should_view_contents() {
            // Just copy the entire contents if viewing into the reader is not required.
            io::copy(reader, writer)?;
        } else {
            // Skips contents if either `skip_head` or `skip_tail` is set.
            if self.opts.skip_head.is_some() || self.opts.skip_tail.is_some() {
                let mut seeker = ByteSeeker::new(reader);

                // Position to start reading.
                seeker.reset();
                let start = match &self.opts.skip_head {
                    None => 0,
                    Some(skip) => match *skip {
                        Skip::Bytes(n) => n,
                        Skip::BytesOnce(n) => match pos {
                            PartPos::Start => 0,
                            _ => n,
                        },
                        Skip::Lines(n) => match n {
                            0 => 0,
                            _ => {
                                let pos;

                                if !endn && n == 1 {
                                    match seeker.seek_nth(b"\n", 1) {
                                        Ok(idx) => {
                                            pos = idx + 1;
                                        }
                                        Err(e) => match e.kind() {
                                            byteseeker::ErrorKind::ByteNotFound => pos = stream_len,
                                            _ => return Err(e.into()),
                                        },
                                    }
                                } else {
                                    let nth = if endn { n } else { n - 1 };
                                    match seeker.seek_nth(b"\n", nth) {
                                        Ok(idx) => {
                                            if endn {
                                                pos = idx + 1;
                                            } else {
                                                match seeker.seek(b"\n") {
                                                    Ok(idx) => {
                                                        pos = idx + 1;
                                                    }
                                                    Err(e) => match e.kind() {
                                                        byteseeker::ErrorKind::ByteNotFound => {
                                                            pos = stream_len
                                                        }
                                                        _ => return Err(e.into()),
                                                    },
                                                }
                                            }
                                        }
                                        Err(e) => match e.kind() {
                                            byteseeker::ErrorKind::ByteNotFound => {
                                                return Err(ErrorKind::InvalidSkip);
                                            }
                                            _ => return Err(e.into()),
                                        },
                                    }
                                }

                                pos
                            }
                        },
                        Skip::LinesOnce(n) => match pos {
                            PartPos::Start => 0,
                            _ => match n {
                                0 => 0,
                                _ => {
                                    let pos;

                                    if !endn && n == 1 {
                                        match seeker.seek_nth(b"\n", 1) {
                                            Ok(idx) => {
                                                pos = idx + 1;
                                            }
                                            Err(e) => match e.kind() {
                                                byteseeker::ErrorKind::ByteNotFound => {
                                                    pos = stream_len
                                                }
                                                _ => return Err(e.into()),
                                            },
                                        }
                                    } else {
                                        let nth = if endn { n } else { n - 1 };
                                        match seeker.seek_nth(b"\n", nth) {
                                            Ok(idx) => {
                                                if endn {
                                                    pos = idx + 1;
                                                } else {
                                                    match seeker.seek(b"\n") {
                                                        Ok(idx) => {
                                                            pos = idx + 1;
                                                        }
                                                        Err(e) => match e.kind() {
                                                            byteseeker::ErrorKind::ByteNotFound => {
                                                                pos = stream_len
                                                            }
                                                            _ => return Err(e.into()),
                                                        },
                                                    }
                                                }
                                            }
                                            Err(e) => match e.kind() {
                                                byteseeker::ErrorKind::ByteNotFound => {
                                                    return Err(ErrorKind::InvalidSkip);
                                                }
                                                _ => return Err(e.into()),
                                            },
                                        }
                                    }

                                    pos
                                }
                            },
                        },
                        Skip::Until(bytes) => match seeker.seek(bytes) {
                            Ok(pos) => pos + bytes.len(),
                            Err(e) => match e.kind() {
                                byteseeker::ErrorKind::ByteNotFound => stream_len,
                                _ => return Err(e.into()),
                            },
                        },
                        Skip::Before(bytes) => match seeker.seek(bytes) {
                            Ok(pos) => pos,
                            Err(e) => match e.kind() {
                                byteseeker::ErrorKind::ByteNotFound => stream_len,
                                _ => return Err(e.into()),
                            },
                        },
                        Skip::Repeats(bytes) => {
                            let width = bytes.len();
                            match width {
                                0 => 0,
                                _ => {
                                    let mut buf = Vec::with_capacity(width);
                                    buf.resize(width, 0);

                                    let mut reader = seeker.get_mut();
                                    util::seek_to_start(&mut reader)?;
                                    let mut bytes_match = 0;
                                    loop {
                                        reader.read_exact(&mut buf)?;
                                        if &buf == bytes {
                                            bytes_match += width;
                                            if bytes_match == stream_len {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }

                                    bytes_match
                                }
                            }
                        }
                        _ => unimplemented!(),
                    },
                };

                // Position to end reading.
                //
                // Only bytes before this position will be read.
                seeker.reset();
                let end = match &self.opts.skip_tail {
                    None => util::seek_to_end(reader)? as usize,
                    Some(skip) => match *skip {
                        Skip::Bytes(n) => match n > stream_len {
                            true => return Err(ErrorKind::InvalidSkip),
                            false => stream_len - n,
                        },
                        Skip::BytesOnce(n) => match pos {
                            PartPos::End => stream_len,
                            _ => match n > stream_len {
                                true => return Err(ErrorKind::InvalidSkip),
                                false => stream_len - n,
                            },
                        },
                        Skip::Lines(n) => match n {
                            0 => stream_len,
                            _ => {
                                let pos;

                                // Ignore any ending newline.
                                if endn {
                                    seeker.seek_back(b"\n")?;
                                }

                                match n {
                                    1 => match seeker.seek_back(b"\n") {
                                        Ok(idx) => {
                                            pos = idx + 1;
                                        }
                                        Err(e) => match e.kind() {
                                            byteseeker::ErrorKind::ByteNotFound => pos = 0,
                                            _ => return Err(e.into()),
                                        },
                                    },
                                    _ => match seeker.seek_nth_back(b"\n", n - 1) {
                                        Ok(_) => match seeker.seek_back(b"\n") {
                                            Ok(idx) => pos = idx + 1,
                                            Err(e) => match e.kind() {
                                                byteseeker::ErrorKind::ByteNotFound => pos = 0,
                                                _ => return Err(e.into()),
                                            },
                                        },
                                        Err(e) => match e.kind() {
                                            byteseeker::ErrorKind::ByteNotFound => {
                                                return Err(ErrorKind::InvalidSkip)
                                            }
                                            _ => return Err(e.into()),
                                        },
                                    },
                                }

                                pos
                            }
                        },
                        Skip::LinesOnce(n) => match pos {
                            PartPos::End => stream_len,
                            _ => match n {
                                0 => stream_len,
                                _ => {
                                    let pos;

                                    // Ignore any ending newline.
                                    if endn {
                                        seeker.seek_back(b"\n")?;
                                    }

                                    match n {
                                        1 => match seeker.seek_back(b"\n") {
                                            Ok(idx) => {
                                                pos = idx + 1;
                                            }
                                            Err(e) => match e.kind() {
                                                byteseeker::ErrorKind::ByteNotFound => pos = 0,
                                                _ => return Err(e.into()),
                                            },
                                        },
                                        _ => match seeker.seek_nth_back(b"\n", n - 1) {
                                            Ok(_) => match seeker.seek_back(b"\n") {
                                                Ok(idx) => pos = idx + 1,
                                                Err(e) => match e.kind() {
                                                    byteseeker::ErrorKind::ByteNotFound => pos = 0,
                                                    _ => return Err(e.into()),
                                                },
                                            },
                                            Err(e) => match e.kind() {
                                                byteseeker::ErrorKind::ByteNotFound => {
                                                    return Err(ErrorKind::InvalidSkip)
                                                }
                                                _ => return Err(e.into()),
                                            },
                                        },
                                    }

                                    pos
                                }
                            },
                        },
                        Skip::Until(bytes) => match seeker.seek_back(bytes) {
                            Ok(pos) => pos,
                            Err(e) => match e.kind() {
                                byteseeker::ErrorKind::ByteNotFound => 0,
                                _ => return Err(e.into()),
                            },
                        },
                        Skip::Before(bytes) => match seeker.seek(bytes) {
                            Ok(pos) => pos + bytes.len(),
                            Err(e) => match e.kind() {
                                byteseeker::ErrorKind::ByteNotFound => 0,
                                _ => return Err(e.into()),
                            },
                        },
                        Skip::Repeats(bytes) => {
                            let width = bytes.len();
                            match width {
                                0 => stream_len,
                                _ => {
                                    let mut buf = Vec::with_capacity(width);
                                    buf.resize(width, 0);

                                    let mut reader = seeker.get_mut();
                                    util::seek_to_end(&mut reader)?;
                                    let mut bytes_match = 0;
                                    loop {
                                        // Avoid seek negative.
                                        if bytes_match + width > stream_len {
                                            break;
                                        }
                                        util::seek_end(-((bytes_match + width) as i64), reader)?;
                                        reader.read_exact(&mut buf)?;
                                        if &buf == bytes {
                                            bytes_match += width;
                                            if bytes_match == stream_len {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }

                                    stream_len - bytes_match
                                }
                            }
                        }
                    },
                };

                match (start, end) {
                    _ if end < start => return Err(ErrorKind::InvalidSkip),
                    _ if start == end => (),
                    _ => {
                        // Reads the desired contents.
                        let bytes_count = end - start;
                        match bytes_count {
                            0 => (),
                            _ => {
                                let mut buf_reader = BufReader::new(reader);
                                let mut read = 0;
                                util::seek_start(start as u64, &mut buf_reader)?;

                                loop {
                                    let buf = buf_reader.fill_buf()?;
                                    let length = buf.len();
                                    if length == 0 {
                                        break;
                                    }
                                    if read + length > bytes_count {
                                        let mut buffer = buf.to_owned();
                                        buffer.truncate(bytes_count - read);
                                        writer.write_all(&buffer)?;
                                        buf_reader.consume(length);
                                    } else {
                                        read += length;
                                        writer.write_all(buf)?;
                                        buf_reader.consume(length);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // Just copy the entire contents of the given reader into the given writer.
                io::copy(reader, writer)?;
            }
        }

        // Should we writer ending newline?
        if self.opts.newline.is_some() && !endn {
            match self.opts.newline.unwrap() {
                Newline::Lf => {
                    writer.write_all(b"\n")?;
                }
                Newline::Crlf => {
                    writer.write_all(b"\r\n")?;
                }
            }
        }

        // Writes padding after this source.
        self.write_padding_after(writer, pos)?;

        Ok(())
    }

    fn write_padding_before<W: Write>(&self, writer: &mut W, pos: PartPos) -> Result<()> {
        if let Some(pad) = &self.opts.padding {
            // Check if padding should be filled before this source.
            match (pad, pos) {
                (Pad::Before(padding), PartPos::Start) => {
                    writer.write_all(padding)?;
                }
                (Pad::Custom(Some(padding), _, _), PartPos::Start) => {
                    writer.write_all(padding)?;
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn write_padding_after<W: Write>(&self, writer: &mut W, pos: PartPos) -> Result<()> {
        if let Some(pad) = &self.opts.padding {
            // Check if padding should be filled before this source.
            match (pad, pos) {
                (Pad::After(padding), PartPos::End) => {
                    writer.write_all(padding)?;
                }
                (Pad::Between(padding), PartPos::Start) => {
                    writer.write_all(padding)?;
                }
                (Pad::Between(padding), PartPos::Inside) => {
                    writer.write_all(padding)?;
                }
                (Pad::Custom(_, Some(padding), _), PartPos::Start) => {
                    writer.write_all(padding)?;
                }
                (Pad::Custom(_, Some(padding), _), PartPos::Inside) => {
                    writer.write_all(padding)?;
                }
                (Pad::Custom(_, _, Some(padding)), PartPos::End) => {
                    writer.write_all(padding)?;
                }
                _ => (),
            }
        }

        Ok(())
    }

    // Returns `true` if viewing into the contents of each part is required.
    fn should_view_contents(&self) -> bool {
        self.opts.newline.is_some()
            || self.opts.skip_head.is_some()
            || self.opts.skip_tail.is_some()
    }
}

/// Simliar to [`RsMerger`] but provides dedicated methods to work with [`Path`]s and [`File`]s.
#[derive(Clone, Debug)]
pub struct FileMerger<'a>(RsMerger<'a>);

impl<'a> Default for FileMerger<'a> {
    fn default() -> Self {
        let opts = RsMergerOptions {
            skip_head: None,
            skip_tail: None,
            padding: None,
            newline: None,
        };
        FileMerger(RsMerger { opts })
    }
}

impl<'a> FileMerger<'a> {
    /// Creates a new `FileMerger` builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Configures this merger to skip partial of contents from the head of each file.
    pub fn skip_head(&mut self, skip: Skip<'a>) -> &mut Self {
        self.0.opts.skip_head = Some(skip);
        self
    }

    /// Configures this merger to skip partial of contents from the tail of each file.
    pub fn skip_tail(&mut self, skip: Skip<'a>) -> &mut Self {
        self.0.opts.skip_tail = Some(skip);
        self
    }

    /// Configures this merger to fill some padding before, between or after the file contents.
    pub fn pad_with(&mut self, padding: Pad<'a>) -> &mut Self {
        self.0.opts.padding = Some(padding);
        self
    }

    /// Configures this merger to force the presence of ending newline after each file.
    pub fn force_ending_newline(&mut self, newline: Newline) -> &mut Self {
        self.0.opts.newline = Some(newline);
        self
    }

    /// Open file paths given and merges file contents into the given writer according to the
    /// given configrations.
    ///
    /// Nothing that this method will return an error if any path given is not point to a regular
    /// file. For an variant that ignores invalid paths, see [`with_paths_lossy`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use admerge::{FileMerger, Skip, Pad, Newline, Result};
    /// use std::fs::OpenOptions;
    ///
    /// fn main() -> Result<()> {
    ///     let mut file = OpenOptions::new().append(true).create(true).open("merged.txt")?;
    ///
    ///     // Configures merger.
    ///     let mut merger = FileMerger::new();
    ///     merger.skip_tail(Skip::LinesOnce(1));
    ///     merger.pad_with(Pad::Before(b"leading contents\n"));
    ///     merger.force_ending_newline(Newline::Lf);
    ///
    ///     // Merges sources into one.
    ///     merger.with_paths(vec!["foo.txt", "bar.txt", "baz.txt"], &mut file)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error variant of [`ErrorKind::NothingPassed`] if the given path vector is
    /// empty;
    ///
    /// Returns an error variant of [`ErrorKind::InvalidPath`] if the given paths contain invalid
    /// path.
    ///
    /// Returns an error variant of [`ErrorKind::InvalidSkip`] if the given [`Skip`]s cannot
    /// applied to the given sources;
    ///
    /// Returns an error variant of [`ErrorKind::Io`] if any I/O errors were encountered.
    ///
    /// [`with_paths_lossy`]: FileMerger::with_paths_lossy
    pub fn with_paths<P, W>(&self, paths: Vec<P>, writer: &mut W) -> Result<()>
    where
        P: AsRef<Path>,
        W: Write,
    {
        let sources: Result<Vec<_>> = paths
            .into_iter()
            .map(|p| File::open(p).map_err(|e| e.into()))
            .collect();

        self.with_files(sources?, writer)
    }

    /// Open every file path given if path points to a regular file, and then merges file contents
    /// into the given writer according to the given configrations.
    ///
    /// See also [`with_paths`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use admerge::{FileMerger, Skip, Pad, Newline, Result};
    /// use std::fs::OpenOptions;
    ///
    /// fn main() -> Result<()> {
    ///     let mut file = OpenOptions::new().append(true).create(true).open("merged.txt")?;
    ///
    ///     // Configures merger.
    ///     let mut merger = FileMerger::new();
    ///     merger.skip_tail(Skip::LinesOnce(1));
    ///     merger.pad_with(Pad::Before(b"leading contents\n"));
    ///     merger.force_ending_newline(Newline::Lf);
    ///
    ///     // Merges sources into one.
    ///     merger.with_paths_lossy(vec!["foo.txt", "bar.txt", "not a file path"], &mut file)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error variant of [`ErrorKind::NothingPassed`] if the given path vector is
    /// empty;
    ///
    /// Returns an error variant of [`ErrorKind::InvalidSkip`] if the given [`Skip`]s cannot
    /// applied to the given sources;
    ///
    /// Returns an error variant of [`ErrorKind::Io`] if any I/O errors were encountered.
    ///
    /// [`with_paths`]: FileMerger::with_paths
    pub fn with_paths_lossy<P, W>(&self, paths: Vec<P>, writer: &mut W) -> Result<()>
    where
        P: AsRef<Path>,
        W: Write,
    {
        // Dumps any path that does not point to a regular file..
        let sources: Vec<_> = paths.into_iter().filter(|p| p.as_ref().is_file()).collect();

        self.with_paths(sources, writer)
    }

    /// Reads sequentially from the given files and merges their contents into the given writer
    /// according to the given configrations.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use admerge::{FileMerger, Skip, Pad, Newline, Result};
    /// use std::fs::{File, OpenOptions};
    ///
    /// fn main() -> Result<()> {
    ///     let f1 = File::open("foo.txt")?;
    ///     let f2 = File::open("bar.txt")?;
    ///     let f3 = File::open("baz.txt")?;
    ///     let mut file = OpenOptions::new().append(true).create(true).open("merged.txt")?;
    ///
    ///     // Configures merger.
    ///     let mut merger = FileMerger::new();
    ///     merger.skip_tail(Skip::LinesOnce(1));
    ///     merger.pad_with(Pad::Before(b"leading contents\n"));
    ///     merger.force_ending_newline(Newline::Lf);
    ///
    ///     // Merges sources into one.
    ///     merger.with_files(vec![f1, f2, f3], &mut file)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error variant of [`ErrorKind::NothingPassed`] if the given path vector is
    /// empty;
    ///
    /// Returns an error variant of [`ErrorKind::InvalidSkip`] if the given [`Skip`]s cannot
    /// applied to the given sources;
    ///
    /// Returns an error variant of [`ErrorKind::Io`] if any I/O errors were encountered.
    pub fn with_files<W>(&self, files: Vec<File>, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        self.0.merge_sources_into(files, writer)
    }
}
