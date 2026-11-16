// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Corrections a speaker makes as they speak ("coffee at 2, actually 3"),
//! applied by a small language model after recognition, with the line
//! breaks and quotation marks the speaker said and no others (`said`), and
//! the check its answer must pass to be used: a model that loses a word the
//! speaker said, or answers with nothing, is not trusted, and the text
//! stands as heard.

mod corrector;
mod draft;
mod kept;
mod prompts;
mod said;

pub use corrector::{CleanupModel, Corrector};
pub use kept::kept;
pub use prompts::{Prompt, GEMMA_4, S1_MINI, SPEAKOFLOW_MINI};
pub use said::as_said;
