# saucepan

Data structures and utilities for dealing with source spans.

Saucepan is a Franken-crate, a mash-up of [codespan](https://crates.io/crates/codespan) and
[nom_locate](https://crates.io/crates/nom_locate). The [nom](https://crates.io/crates/nom)
dependency is *optional* and can be disabled. Unlike nom_locate, saucepan does not have no-std
support. It does include implementations of `File` and `Files` (as `Source` and `Sources` resp.),
which are compatible with [codespan-reporting](https://crates.io/crates/codespan-reporting).


![saucepan](saucepan.png)


# Is saucepan right for you?

You can probably appreciate the attraction to a single span struct having the advantages of both
`codespan::span` and `LocatedSpan`. But before using this crate in your project you should evaluate
whether saucepan is the right choice for you. This is a Franken-crate in a second sense: It looks
like a pile of dead body parts sewn together. If you only need nom_locate or codespan, you probably
shouldn't use this crate. If you are interested in something more feature full (see below) and are
willing to contribute some coding effort into making it work for your use case (PRs welcome!),
saucepan might be for you. 

This crate should not be used in production at this time. Potential contributors should look at the
TODO.md file and the `// todo` comments if they want guidance on what to improve.

# ...why?

The purpose of this crate is to have a single *thing* that can take the place of both
 `nom_locate::LocatedSpan`
 and `codespan::Span`. That thing is called `Span` in this crate. A span can
 
 * be used as an input/output type for `nom` parser combinators
 * be queried for the byte offset, line (row) number, and column number of the text it represents
 * provide a `&str` of the text it represents
 * keep track of the file handle of the original source file the text is from
 * be provided to a `Source` or `Sources` instance to retrieve the name of the source file

The `Span` struct itself implements copy:

```rust
pub struct Span<SourceType> {
  /// This `Span` begins at byte `start` in the original "Source"
  start: ByteIndex,     // 32 bits
  /// A text fragment is usually a slice of the original source text, e.g. `&str` or `&[u8]`.
  fragment: SourceType, // twice the size of a pointer to a 
                        // `Sized` object.
  /// A handle to the file from which the text comes.
  pub(crate) source_id: SourceID // 32 bits
}
```

The `Span` struct is still in flux. In particular, the `source_id` will likely be removed, as the
source file can be derived from the other two data members by its parent `Source`/`Sources `
instance.

This crate is part of a much larger collection of generic scanning and parsing tools in development
and is still a bit of a work in progress.

# authors

As two different projects were used to make a third, authorship needs clarification. The authors
 are as follows.

| saucepan mash-up     | Robert Jacobson         |
| :------------------- | :---------------------- |
|                      |                         |
| codespan             | Brendan Zabarauskas     |
|                      |                         |
| nom_locate           | Florent FAYOLLE         |
|                      | Christopher Durham      |
|                      | Valentin Lorentz        |




