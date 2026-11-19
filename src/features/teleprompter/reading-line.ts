// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

/** Where the word being read stands, as a share of the text frame's
 * height: high, so the eyes stay near the camera above it. */
export const READING_LINE = 0.3;

/** From the reading line down to the middle of the words on it, in points:
 * the word being read hangs from the line. */
export const readingLineOffset = (textSize: number) => textSize * 0.65;

/** The middle of the words being read, down the text frame, as CSS. */
export const readingLineMiddle = (textSize: number) =>
  `calc(${String(READING_LINE * 100)}% + ${String(readingLineOffset(textSize))}px)`;
