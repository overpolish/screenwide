// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TextStyle } from "../../../../../bindings/TextStyle";
import type { TextTemplates } from "../../../recording/captions/use-text-templates";

/** The template a text takes its look from until another is chosen. */
const LABEL = "label";

/**
 * The look a text panel shows: the text's own, or for the next text the one
 * last chosen, or else the Label look in the tool's colour, as Rust dresses
 * a fresh text that has none.
 */
export const shownTextLook = (
  look: TextStyle | undefined,
  color: string,
  templates: TextTemplates,
): TextStyle | null => {
  if (look) return look;
  const label = templates.shipped.find((style) => style.template === LABEL);
  if (!label) return null;
  return label.background === "box"
    ? { ...label, backgroundColor: color }
    : { ...label, textColor: color };
};
