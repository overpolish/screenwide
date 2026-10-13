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
    highlight: t("editor-panels-captions-template-highlight"),
    pop: t("editor-panels-captions-template-pop"),
    reveal: t("editor-panels-captions-template-reveal"),
  })[template];

/** Whether `style` can colour the word being said: one that sets it on a
 * pill marks it already. */
export const offersWordHighlight = (style: CaptionStyle) =>
  style.background !== "word";

/** A colour a caption style is drawn in, by what it colours. */
export type CaptionColorRole = "background" | "highlight" | "text";

/** The style's field each colour is kept in. */
export const CAPTION_COLOR_KEYS = {
  background: "backgroundColor",
  highlight: "highlightColor",
  text: "textColor",
} as const satisfies Record<CaptionColorRole, keyof CaptionStyle>;

/** The colours `style` draws in, so the ones it offers to change: its type
 * always, the word being said where it is marked, and its box where it has
 * one. */
export const offeredColors = (style: CaptionStyle): CaptionColorRole[] => [
  "text",
  ...(style.background === "word" || style.highlightWords
    ? (["highlight"] as const)
    : []),
  ...(style.background === "box" ? (["background"] as const) : []),
];

/** A colour's name, as its pill shows it. */
export const captionColorLabel = (role: CaptionColorRole) =>
  ({
    background: t("editor-panels-captions-color-background"),
    highlight: t("editor-panels-captions-color-highlight"),
    text: t("editor-panels-captions-color-text"),
  })[role];
