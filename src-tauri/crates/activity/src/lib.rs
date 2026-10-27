// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What happens over a recording, read from its picture, for the editor to
//! tell what must play at its own pace from what can be hurried or cut.
//!
//! The cursor and key files say what was done and the transcript what was
//! said; neither says what the picture did by itself: a page loading, a log
//! scrolling, a game. Ten times a second the picture is shrunk to a small
//! grey frame, and each is measured against the ones before it (`measure`):
//! how much of it changed since the last, and how much is unlike anything in
//! the seconds before, which tells a new screen from one that only moves.
//! What that means is judged against the recording's own level (`states`).
//!
//! The capture reads its frames as it writes them (`tap`, `grey`), on every
//! platform alike, so a recording's readings are ready when it stops. A
//! recording made before that is read by decoding it (`sample`).

mod grey;
mod measure;
mod sample;
mod states;
mod tap;
#[cfg(test)]
mod tests;

pub use grey::{grey_from_bgra, grey_from_luma};
pub use measure::{Measure, PictureReading, PictureState, SAMPLE_SIZE};
pub use sample::{read_picture, SAMPLES_PER_SECOND};
pub use states::states;
pub use tap::{read_readings, readings_path, PictureTap};
