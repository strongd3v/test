//! Local direct-link framing: one terminal's I/O over a peer-to-peer stream.
//!
//! [`link`] defines the length-prefixed binary frames both ends speak, with
//! the host serving its own terminals and the viewer attaching, numbering
//! input and reading ordered output.

pub mod link;
