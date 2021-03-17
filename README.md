# admerge

[![Crates.io](https://img.shields.io/crates/v/admerge?style=flat-square)](https://crates.io/crates/admerge)
[![Docs](https://docs.rs/admerge/badge.svg)](https://docs.rs/admerge/)

A Rust library for merging files or in-memory buffers, featuring:

- Easy file merging via [`FileMerger`](https://docs.rs/admerge/*/admerge/struct.FileMerger.html).
- Easy in-memory buffer merging via [`RsMerger`](https://docs.rs/admerge/*/admerge/struct.RsMerger.html).
- Skip unwanted contents of each merge unit from either start or end.
- Fill paddings before, between and/or after each merge unit.
- Force presences of ending newlines after each merge unit.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
admerge = "0.1"
```

To get started using `admerge`, see [documentation](https://docs.rs/admerge/).

For an out-of-box cli that backed by this crate, see [fcc](https://github.com/mapkts/fcc).

## License

Admerge is distributed under the terms of the MIT license.

See [LICENSE-MIT](LICENSE-MIT), and [COPYRIGHT](COPYRIGHT) for details.
