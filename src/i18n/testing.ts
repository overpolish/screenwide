// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Text from `t` without the invisible direction isolates (U+2068, U+2069)
 * Fluent sets each placeholder apart with, so a name in one script reads
 * correctly inside a sentence in another. For tests, which compare text in
 * the source language.
 */
export const withoutIsolates = (text: string) =>
  text.replace(/[\u2068\u2069]/gu, "");
