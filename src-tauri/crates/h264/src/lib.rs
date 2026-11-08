// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! H.264 on Windows: BGRA frames on the GPU converted to NV12 and encoded by
//! the graphics card's own encoder, then written to MP4.
//!
//! The encoder transform is driven here rather than left to a Sink Writer.
//! A Sink Writer that makes a hardware encoder never lets it go: each one
//! kept its Direct3D device, its surfaces and the driver's threads until the
//! app quit. Driven here, it is shut down through the activation object that
//! made it, and everything it held goes with it. The Sink Writer only muxes,
//! given H.264 in and H.264 out.

pub mod annexb;
#[cfg(target_os = "windows")]
mod convert;
#[cfg(target_os = "windows")]
mod encoder;
#[cfg(target_os = "windows")]
mod file;
#[cfg(target_os = "windows")]
mod mf;
#[cfg(target_os = "windows")]
mod mp4;

#[cfg(target_os = "windows")]
pub use convert::{Converter, Crop, SLOTS};
#[cfg(target_os = "windows")]
pub use encoder::{EncodedFrame, Encoder, Picture, RateControl, Settings};
#[cfg(target_os = "windows")]
pub use file::FileEncoder;
#[cfg(target_os = "windows")]
pub use mp4::Mp4Writer;
