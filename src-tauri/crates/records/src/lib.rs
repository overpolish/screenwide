// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The tracks a recording writes beside its movie, one record per line: where
//! the cursor was and what it looked like, and the keys pressed. The recorder
//! writes them; the editor and the export read them back.

pub mod cursor;
pub mod keyboard;
