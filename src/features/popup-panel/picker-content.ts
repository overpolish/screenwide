// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TextStyle } from "../../bindings/TextStyle";
import type { TextTemplateId } from "../../bindings/TextTemplateId";

/** The templates to choose from, as tiles of their moving previews, held
 * on their stills where `still` says the text never moves. `styles` are each
 * as choosing it would set the text; `names` are their manifests' names. */
export type PopupPanelTextTemplatesContent = {
  kind: "text-templates";
  names: Partial<Record<TextTemplateId, string>>;
  selected: TextTemplateId;
  styles: TextStyle[];
  still?: boolean;
};

/** A colour grid: the palette, `savedColors`, and Custom. */
export type PopupPanelColorsContent = {
  kind: "colors";
  savedColors: string[];
  value: string;
};
