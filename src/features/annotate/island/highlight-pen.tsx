// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { t } from "../../../i18n/i18n";
import { IslandKeyHint } from "../../../primitives/island/island-key-hint";
import { HandDrawnToggle } from "../../annotations/options/hand-drawn-toggle";
import { TintToggle } from "../../annotations/options/tint-toggle";

import type { SectionProps } from "./section-props";

/** The highlight's pen: how it is drawn, clean or by hand, and whether it
 * tints what it covers rather than recolouring it, for pages the recolour
 * misreads; both are looks kept from one highlight to the next. Whether it
 * fits the text or covers the box dragged is decided by the drag itself,
 * Option (Alt on Windows) held through it laying the other, so the note
 * beside the choices names the key as a keycap. */
export function HighlightPen({ onChange, settings }: SectionProps) {
  return (
    <div className="flex items-center gap-separate">
      <HandDrawnToggle
        isHandDrawn={settings.highlightHandDrawn}
        isLabelHidden
        kind="highlight"
        onChange={(highlightHandDrawn) => {
          onChange({ highlightHandDrawn });
        }}
      />
      <TintToggle
        isLabelHidden
        isTinted={settings.highlightTint}
        onChange={(highlightTint) => {
          onChange({ highlightTint });
        }}
      />
      {/* Laid as `SectionHint` is: from the toggle, ending short of the
            section's edge. */}
      <IslandKeyHint
        binding="Alt"
        className="max-w-48 ps-0"
        message={(key) =>
          settings.highlightManual
            ? t("annotate-island-highlight-box-hint", { key })
            : t("annotate-island-highlight-fit-hint", { key })
        }
      />
    </div>
  );
}
