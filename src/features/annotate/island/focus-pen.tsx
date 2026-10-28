// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { CornerChoice } from "../../annotations/options/corner-choice";
import { DimToggle } from "../../annotations/options/dim-toggle";

import type { SectionProps } from "./section-props";

/** The focus's pen: its corners, then how it treats what lies around its
 * box. One tool for both looks, as both point at one box: the blur is the
 * look, and the dim keeps whatever moves behind the overlay live, where a
 * blur can only soften a still of the screen. */
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
        <DimToggle
          dims={settings.focusDims}
          onChange={(focusDims) => {
            onChange({ focusDims });
          }}
        />
      </div>
    </IslandCrossfade>
  );
}
