// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { IslandChoice } from "../../../primitives/island/island-choice";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";

import { CornerChoice } from "./corner-choice";
import { FocusLookGlyph } from "./focus-glyphs";

import type { SectionProps } from "./section-props";

const LOOKS = [
  { id: "blur", label: () => t("annotate-island-focus-blur") },
  { id: "dim", label: () => t("annotate-island-focus-dim") },
] as const;

/** The focus's pen: its corners, then how it treats what lies around its
 * box, blurred as a camera's depth of field or dimmed as a spotlight's shade.
 * One tool for both, as both point at one box: the blur is the look, and the
 * dim keeps whatever moves behind the overlay live, where a blur can only
 * soften a still of the screen. Blurred draws a focus, dimmed a spotlight. */
export function FocusPen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex gap-separate">
        <CornerChoice
          onChange={(focusRadius) => {
            onChange({ focusRadius });
          }}
          value={settings.focusRadius}
        />
        <IslandChoice
          items={LOOKS.map(({ id, label }) => ({
            content: <FocusLookGlyph look={id} />,
            id,
            label: label(),
          }))}
          label={t("annotation-outside")}
          onChange={(id) => {
            onChange({ focusDims: id === "dim" });
          }}
          value={settings.focusDims ? "dim" : "blur"}
        />
      </div>
    </IslandCrossfade>
  );
}
