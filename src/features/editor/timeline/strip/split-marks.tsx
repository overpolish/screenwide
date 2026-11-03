// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { TimelineMark } from "../timeline-mark";
import { at, type TimelineScale } from "../timeline-scale";

// Where the scissors split the recording without cutting anything: a place
// something moved can land, inside a beat as well as between them. Its mark
// stands at the top of the beats, as a cut's seam does, with the scissors
// that made it; pressed, it takes the split away.

export function SplitMarks({
  onRemove,
  scale,
  splits,
}: {
  /** Told a split pressed, by the recorded moment it is at. */
  onRemove: (at: number) => void;
  scale: TimelineScale;
  splits: number[];
}) {
  return splits.map((split) => (
    <TimelineMark
      end="top"
      glyph={<Scissors aria-hidden />}
      key={split}
      label={t("editor-timeline-remove-split")}
      onPress={() => {
        onRemove(split);
      }}
      style={at(scale, split)}
    />
  ));
}
