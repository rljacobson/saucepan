# codespan

[![Continuous integration][actions-badge]][actions-url]
[![Crates.io][crate-badge]][crate-url]
[![Docs.rs][docs-badge]][docs-url]
[![Gitter][gitter-badge]][gitter-lobby]

[actions-badge]: https://img.shields.io/github/workflow/status/brendanzab/codespan/Continuous%20integration
[actions-url]: https://github.com/brendanzab/codespan/actions
[crate-url]: https://crates.io/crates/codespan
[crate-badge]: https://img.shields.io/crates/v/codespan.svg
[docs-url]: https://docs.rs/codespan
[docs-badge]: https://docs.rs/codespan/badge.svg
[gitter-badge]: https://badges.gitter.im/codespan-rs/codespan.svg
[gitter-lobby]: https://gitter.im/codespan-rs/Lobby

Utilities for dealing with source code locations.

Eventually this crate will be deprecated in favor of `codespan_reporting` only. This crate is a stop-gap providing minimal concrete implementations of Span, File, and Files. The `codespan_reporting` and `codespan_lsp` project files have been removed, and some superficial renaming and modifications have been done.