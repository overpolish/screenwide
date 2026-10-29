// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { IslandKeyHint } from "../../../primitives/island/island-key-hint";
import { HandDrawnToggle } from "../../annotations/options/hand-drawn-toggle";

import type { SectionProps } from "./section-props";

/** The highlight's pen: only how it is drawn, clean or by hand, which is a
 * look kept from one highlight to the next. Whether it fits the text or
 * covers the box dragged is decided by the drag itself, Option (Alt on
 * Windows) held through it laying the other, so the note beside the choice
 * names the key as a keycap; whether it recolours or tints is a fallback for
 * pages recolouring misreads, left to Settings. */
export function HighlightPen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex items-center gap-separate">
        <HandDrawnToggle
          isHandDrawn={settings.highlightHandDrawn}
          kind="highlight"
          onChange={(highlightHandDrawn) => {
            onChange({ highlightHandDrawn });
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
    </IslandCrossfade>
  );
}
