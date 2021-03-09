//! Provides mergers with advanced options.
//!
//! The main entities of this crate are [`RsMerger`] and [`FileMerger`]. The former works on any
//! source that implemnts [`Read`] and [`Seek`] traits; the latter one is mostly identical with the
//! former, but provides addtional methods to work with [`Path`]s and [`File`]s.
//!
//! # Behaviours
//!
//! When merging sources, mergers provided by this crate allow you to skip partials of contents
//! from each source, pad with extra padding between sources. No modifications are done to the
//! given sources as it violate the semantics of merging.
//!
//! The current algorithm to merge sources is described as following:
//!
//! 1. If any leading padding is given by [`pad_with`], writes it into the writer.
//! 2. For each source, if skip range is given by [`skip_head`] and
//!    [`skip_tail`], writes the contents remain into the writer.
//! 3. For each source, if forcing ending newline option is set by
//!    [`force_ending_newline`], writes a ending newline into the writer if this source
//!    is not ends with a newline.
//! 4. For each source, if any inner padding is given by [`pad_with`], writes it into the
//!    writer.
//! 5. If any ending padding is given by [`pad_with`], writes it into the writer.
//!
//! When merging three sources, the result should be imagined as following:
//!
//! ```bash
//! ------------------------------
//! |       Padding before       |  <== `Pad::Before` apprears here.
//! ------------------------------
//! ------------------------------
//! |     Source 1 (skipped)     |  <== `skip_head` ignores partials of contents of source 1.
//! ------------------------------
//! ------------------------------
//! |    Source 1 (remaining)    |  <== The remaining contents of source 1.
//! ------------------------------
//! ------------------------------
//! |     Source 1 (skipped)     |  <== `skip_tail` ignores partials of contents of source 1.
//! ------------------------------
//! ------------------------------
//! |      An ending newline     |  <== `force_ending_newline` appends a newline here.
//! ------------------------------
//! ------------------------------
//! |       Padding between      |  <== `Pad::Between` apprears here.
//! ------------------------------
//! ------------------------------
//! |     Source 2 (skipped)     |  <== `skip_head` ignores partials of contents of source 2.
//! ------------------------------
//! ------------------------------
//! |    Source 2 (remaining)    |  <== The remaining contents of source 2.
//! ------------------------------
//! ------------------------------
//! |     Source 2 (skipped)     |  <== `skip_tail` ignores partials of contents of source 2.
//! ------------------------------
//! ------------------------------
//! |      An ending newline     |  <== `force_ending_newline` appends a newline here.
//! ------------------------------
//! ------------------------------
//! |       Padding between      |  <== `Pad::Between` apprears here.
//! ------------------------------
//! ------------------------------
//! |     Source 3 (skipped)     |  <== `skip_head` ignores partials of contents of source 3.
//! ------------------------------
//! ------------------------------
//! |    Source 3 (remaining)    |  <== The remaining contents of source 3.
//! ------------------------------
//! ------------------------------
//! |     Source 3 (skipped)     |  <== `skip_tail` ignores partials of contents of source 3.
//! ------------------------------
//! ------------------------------
//! |      An ending newline     |  <== `force_ending_newline` appends a newline here.
//! ------------------------------
//! ------------------------------
//! |       Padding after        |  <== `Pad::After` apprears here.
//! ------------------------------
//! ```
//!
//! [`Read`]: std::io::Read
//! [`Seek`]: std::io::Seek
//! [`Path`]: std::path::Path
//! [`File`]: std::fs::File
//! [`new`]: RsMerger::new
//! [`pad_with`]: RsMerger::pad_with
//! [`skip_head`]: RsMerger::skip_head
//! [`skip_tail`]: RsMerger::skip_tail
//! [`force_ending_newline`]: RsMerger::force_ending_newline
//! [`merge_sources_into`]: RsMerger::merge_sources_into
mod error;
mod merge;
mod util;

pub use error::*;
pub use merge::*;
