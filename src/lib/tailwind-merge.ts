// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { extendTailwindMerge } from "tailwind-merge";

/**
 * Semantic font sizes declared as `--text-*` in `index.css`. tailwind-merge
 * treats unknown `text-*` classes as colours, so without this the size class
 * gets dropped whenever a text colour follows it.
 */
const fontSizes = [
  "title",
  "headline",
  "body",
  "subheadline",
  "label",
  "footnote",
];

export const twMergeConfig = {
  extend: { theme: { text: fontSizes } },
};

export const twMerge = extendTailwindMerge(twMergeConfig);
