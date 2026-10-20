// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { extendTailwindMerge } from "tailwind-merge";

/**
 * Semantic font sizes declared as `--text-*` in `styles/theme.css`. tailwind-merge
 * treats unknown `text-*` classes as colours, so without this the size class
 * gets dropped whenever a text colour follows it.
 */
const fontSizes = ["title", "heading", "body", "label", "caption"];

/**
 * Semantic spacing declared as `--spacing-*` in `styles/theme.css`. Without these,
 * two sizing classes on different tokens are both kept and CSS order decides.
 */
const spacing = [
  "control-height",
  "part-height",
  "icon-sm",
  "icon",
  "icon-lg",
  "inline",
  "separate",
  "inset",
  "inset-sm",
  "traffic-lights",
];

/**
 * Semantic radii declared as `--radius-*` in `styles/theme.css`. Without these, two
 * `rounded-*` classes on different tokens are both kept and CSS order decides
 * which corner a control ends up with.
 */
const radii = ["part", "control", "panel"];

/**
 * An axis class covers both logical sides, as it covers both physical ones:
 * `px-4` passed to a component overrides its own `ps-2`. tailwind-merge only
 * knows the physical pairs.
 */
const conflictingClassGroups = {
  "inset-x": ["start", "end"],
  mx: ["ms", "me"],
  px: ["ps", "pe"],
  "scroll-mx": ["scroll-ms", "scroll-me"],
  "scroll-px": ["scroll-ps", "scroll-pe"],
} as const;

export const twMergeConfig = {
  extend: {
    conflictingClassGroups,
    theme: { radius: radii, spacing, text: fontSizes },
  },
};

export const twMerge = extendTailwindMerge(twMergeConfig);
