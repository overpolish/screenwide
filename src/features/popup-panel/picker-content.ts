// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CaptionStyle } from "../../bindings/CaptionStyle";
import type { CaptionTemplateId } from "../../bindings/CaptionTemplateId";

/** The caption templates to choose from, as tiles of their moving previews.
 * `styles` are each as choosing it would set the captions; `names` are
 * their manifests' names. */
export type PopupPanelCaptionTemplatesContent = {
  kind: "caption-templates";
  names: Partial<Record<CaptionTemplateId, string>>;
  selected: CaptionTemplateId;
  styles: CaptionStyle[];
};

/** A colour grid: the palette, `savedColors`, and Custom. */
export type PopupPanelColorsContent = {
  kind: "colors";
  savedColors: string[];
  value: string;
};
