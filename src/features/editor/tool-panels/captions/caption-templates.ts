// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplate } from "../../../../bindings/CaptionTemplate";

/** A caption template's name. */
export const captionTemplateLabel = (template: CaptionTemplate) =>
  ({
    box: t("editor-panels-captions-template-box"),
    clean: t("editor-panels-captions-template-clean"),
    editorial: t("editor-panels-captions-template-editorial"),
    highlight: t("editor-panels-captions-template-highlight"),
    karaoke: t("editor-panels-captions-template-karaoke"),
    pop: t("editor-panels-captions-template-pop"),
    punch: t("editor-panels-captions-template-punch"),
    stack: t("editor-panels-captions-template-stack"),
    staircase: t("editor-panels-captions-template-staircase"),
    supersize: t("editor-panels-captions-template-supersize"),
  })[template];

/** Whether `style` can outline its type: one set on a box reads without,
 * and an outline over the box only thickens the type. */
export const offersOutline = (style: CaptionStyle) =>
  style.background !== "box";

/** A colour a caption style is drawn in, by what it colours. */
export type CaptionColorRole =
  "accent" | "background" | "highlight" | "outline" | "text";

/** The style's field each colour is kept in. */
export const CAPTION_COLOR_KEYS = {
  accent: "accentColor",
  background: "backgroundColor",
  highlight: "highlightColor",
  outline: "outlineColor",
  text: "textColor",
} as const satisfies Record<CaptionColorRole, keyof CaptionStyle>;

/** The colours `style` draws in, so the ones it offers to change: its type
 * always, the word being said where it is marked, the accent where it sets
 * words apart, its outline where it has one, and its box where it has one. */
export const offeredColors = (style: CaptionStyle): CaptionColorRole[] => [
  "text",
  ...(style.highlightWords ? (["highlight"] as const) : []),
  ...(style.accent === "none" ? [] : (["accent"] as const)),
  ...(style.outline && offersOutline(style) ? (["outline"] as const) : []),
  ...(style.background === "box" ? (["background"] as const) : []),
];

/** A colour's name, as its pill shows it. */
export const captionColorLabel = (role: CaptionColorRole) =>
  ({
    accent: t("editor-panels-captions-color-accent"),
    background: t("editor-panels-captions-color-background"),
    highlight: t("editor-panels-captions-color-highlight"),
    outline: t("editor-panels-captions-color-outline"),
    text: t("editor-panels-captions-color-text"),
  })[role];
