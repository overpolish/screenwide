// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation: the microphone heard as it is spoken. A session opens the
//! microphone, brings it to what the speech model hears (`resample`) and
//! hands it a little at a time to dictation's helper, which streams it
//! through the model, or in a language no model streams well, writes each
//! stretch once a pause ends it, and answers with the text so far. Spoken
//! commands, in the dictation's language, are carried out on that text
//! (`commands`). Pasting the text is `screenwide-paste`'s.

mod commands;
mod resample;
mod session;

pub use commands::{phrases, shape, Does, Phrases, Shaped, WordChange};
pub use screenwide_speech::runner::dictation::DictatedText;
pub use session::{AtPauses, Ended, Event, Session, Setup};
