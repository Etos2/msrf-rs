# Multi-Source Record Format (MSRF)

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

MSRF is a TLV-esque file format created to store data from different sources into a single binary format in a stream friendly format.
The container format consists of a header and records which encapsulate the raw binary data alongside an identifier of it's source and what type of data is contained.

The specification is defined in the [wiki](https://github.com/Etos2/msrf-rs/wiki).

## Reader/ Writer
This library includes a reader and writer adhering to the specification written in Rust.
Currently, it is heavily WIP and as such no examples are currently present.
