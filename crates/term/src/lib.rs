//! The terminal emulator core shared by the desktop panel and native views
//! apps: [`emulator`] folds PTY bytes into a grid; [`predict`] paints typing
//! ahead of a remote host's echo once the host has proven it echoes.

pub mod emulator;
pub mod predict;
