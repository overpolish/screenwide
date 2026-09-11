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
  "section",
  "footnote",
];

/**
 * Semantic spacing declared as `--spacing-*` in `index.css`. Without these,
 * two padding classes on different tokens are both kept and CSS order decides.
 */
const spacing = [
  "tight",
  "control",
  "control-inset",
  "section",
  "layout",
  "window-inset",
  "control-height",
  "timeline-gutter",
  "traffic-lights",
  "focus-safe",
  "icon",
  "icon-small",
  "icon-mini",
  "icon-large",
  "icon-xl",
];

/**
 * Semantic radii declared as `--radius-*` in `index.css`. Without these, two
 * `rounded-*` classes on different tokens are both kept and CSS order decides
 * which corner a control ends up with.
 */
const radii = ["window", "panel", "control"];

export const twMergeConfig = {
  extend: { theme: { radius: radii, spacing, text: fontSizes } },
};

export const twMerge = extendTailwindMerge(twMergeConfig);
