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

/** `look` as a text placed by hand takes it up: its words come on together
 * until Word by word is chosen, whatever the template does for captions. */
export const asTitleLook = (look: TextStyle): TextStyle => ({
  ...look,
  revealWords: false,
});

/** The templates a text placed by hand is offered, each as it would take
 * it up. One that rolls a few words through at a time follows speech, so
 * it is for captions alone. */
export const titleTemplates = (templates: TextTemplates): TextTemplates => ({
  ...templates,
  styles: templates.styles
    .filter((style) => style.visibleWords === 0)
    .map(asTitleLook),
});
