# saucepan

Data structures and utilities for dealing with source spans.


![saucepan](saucepan.png)

##  Overview

### Features

The purpose of this crate is to have a single *thing* that can take the place of both
`nom_locate::LocatedSpan` and `codespan::Span`. That thing is called a span (of type `Span`) in this
crate. A span can

 * be used as an input/output type for `nom` parser combinators
 * be queried for the byte offset, line (row) number, and column number of the text it represents
 * provide a `&str` of the text it represents
 * keep track of the file handle of the original source file the text is from
 * support a mechanism to retrieve the name of the source file
 * be lightweight, implement `Copy`
 * be nonallocating
 * intgrate with `codespan_reporting` or equivalent

For now the primitive input data type can be either `&str` or `&[u8]`. It would also be nice to have
generic input type, but getting the trait bounds right is nontrivial. Maybe in the future.

#### difference from existing libraries

Before writing a new library, I surveyed the existing options to see if one met my needs. My
requirements are as follows:

 * integration with error reporting (`codespan_reporting` or equivalent)
 * lightweight span
 * nonallocating
 * span can be input type (i.e. gives access to slice)
 * span can also give `(row, col)` info
 * unambiguous multi-file support

The alternatives are:

1. [`nom_locate`](https://crates.io/crates/`nom_locate`)
    1. Excellent integration with `nom`.
    2. generic input type
    3. minimal location types
    4. Includes `Source`/`Sources` types
    5. _No support for reporting_
    6. _lots of unsafe_ (a requirement of its design decisions)
2. [`codespan`](https://crates.io/crates/codespan)
    1. Native integration with `codespan_reporting`
    2. lightweight
    3. _no buffer/slice, thus cannot be used as input_
    4. Otherwise full-featured.
3. [`source-span`](https://crates.io/crates/source-span)
    1. Very nice, full-featured. Includes:...
    2. error reporting
    3. Text buffer
    4. Location types
    5. _allocating_
4. [`rls-span`](https://crates.io/crates/rls-span)
    1. Excellent for its purpose
    2. _Way_ more abstractions than necessary (`ZeroIndexed`, `OneIndexed` subtraits of `Indexed`)
    3. _Poorly documented_
    4. _No slice/buffer — location only_
    5. _No error reporting_




### Is saucepan right for you?

#### saucepan's use case

Saucepan's use case is a situation in which a single type needs to serve as both an input slice type
compatible with `nom` and a span type (potentially compatible with `codespan_reporting` or
equivalent).

#### when to use something else

If you only need `nom_locate` or `codespan`, or if your application can just as easily use both
`nom_locate` and `codespan` to satisfy its needs, you probably shouldn't use this crate, as they are
much more mature and possibly slightly easier to work with.


This crate should not be used in production at this time. Potential contributors should look
at the TODO.md file and the `// todo` comments if they want guidance on what to improve.

## Using Saucepan

### Feature Flags

| Feature Flag            | Description                                                  |
| ----------------------- | ------------------------------------------------------------ |
| `reporting`             | Enable conversion to native codespan object                  |
| `generic-simd`          | Corresponds to `bytecount/generic-simd`                      |
| `runtime-dispatch-simd` | Corresponds to `bytecount/runtime-dispatch-simd`             |
| `nom-parsing`           | Enable conversions for native `nom_locate` objects, use of `Span` as an input for Nom |
| `serialization`         | Enable `serde` serialization support                         |


The default feature set is `["reporting", "nom-parsing", "runtime-dispatch-simd"] `

### Quick Start

The highest level structure in Saucepan is the `Sources` struct, which represents a collection of sources of a
single text. The idea is that a single text can be made up of multiple files in the filesystem. For example, one
source file might "include" the contents of another source file. The `Sources` struct owns one or more `Source`
instances. `Sources` has not yet evolved into its full potential and currently only exists to house a bunch of
boilerplate making it compatible with the `reporting` crate. It is often easier to use the `Source` struct directly.





## Authors and License

Copyright (C) Robert Jacobson

Released under the
Apache License Version 2.0. See [LICENSE](LICENSE) for details.

This work may contain code derived from `codespan`, which is published under the Apache 2.0 license
(see the file LICENSE), and `nom_locate`, which is published under the MIT license (see the file
LICENSE_MIT).
