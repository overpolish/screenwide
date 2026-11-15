// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation: the microphone heard as it is spoken. A session opens the
//! microphone, brings it to what the speech model hears (`resample`) and
//! hands it a little at a time to dictation's helper, which streams it
//! through the model and answers with the text so far. Pasting the text is
//! `screenwide-paste`'s.

mod resample;
mod session;

pub use screenwide_speech::runner::dictation::DictatedText;
pub use session::{Ended, Event, Session};
