// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { splitAtPlaceable } from "../../../i18n/split-at-placeable";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { Keyboard } from "../../../primitives/keyboard/keyboard";
import { HandDrawnToggle } from "../../annotations/options/hand-drawn-toggle";

import { SectionHint } from "./section-hint";

import type { SectionProps } from "./section-props";

/** The key held through a drag to lay the highlight the other way, by the
 * name `Keyboard` reads: drawn as ⌥ on macOS and named Alt on Windows. */
const BOX_KEY = "Alt";

/** The highlight's pen: only how it is drawn, clean or by hand, which is a
 * look kept from one highlight to the next. Whether it fits the text or
 * covers the box dragged is decided by the drag itself, Option (Alt on
 * Windows) held through it laying the other, so the note beside the choice
 * names the key as a keycap; whether it recolours or tints is a fallback for
 * pages recolouring misreads, left to Settings. */
export function HighlightPen({ onChange, ref, settings }: SectionProps) {
  const [before, after] = splitAtPlaceable((key) =>
    settings.highlightManual
      ? t("annotate-island-highlight-box-hint", { key })
      : t("annotate-island-highlight-fit-hint", { key }),
  );
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
        <SectionHint>
          {before}
          <Keyboard className="align-middle">{BOX_KEY}</Keyboard>
          {after}
        </SectionHint>
      </div>
    </IslandCrossfade>
  );
}
