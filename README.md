# Multiple Codec TLVG Container (MCTC)
MCTC is a TLV-esque file format, created with the need to store data from different sources into a single format in a stream friendly format.
The container format consists of a header, a table defining included codecs, and binary data stored in a TLVG encoding (Tag -> Length -> Value -> Guard).

The specification is defined in the wiki.

## Reader/ Writer
This library includes a reader and writer adhering to the specification written in Rust.
Currently, it is heavily WIP and as such no examples are currently present.
