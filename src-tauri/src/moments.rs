// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moments as the app wires them in: a kind's shortcut is claimed only while
//! a recording runs (`shortcuts`), and Settings reads and changes the kinds
//! through `settings`. What a moment is, how it is written beside a
//! recording and how its voice note is taken are the `screenwide-moments`
//! crate's.

pub(crate) mod settings;
pub(crate) mod shortcuts;
