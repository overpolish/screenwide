// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

import { t } from "../../../../i18n/i18n";
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
    <button
      aria-label={t("editor-timeline-remove-split")}
      className="group absolute inset-y-0 z-10 flex w-4 -translate-x-1/2 cursor-default items-start justify-center outline-none"
      dir="ltr"
      key={split}
      onClick={() => {
        onRemove(split);
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={at(scale, split)}
      type="button"
    >
      <span className="absolute inset-y-0 w-px bg-fg-secondary" />
      <span className="relative -mt-1 flex size-4 items-center justify-center rounded-full bg-fg text-surface group-focus-visible:outline-2 group-focus-visible:outline-offset-1 group-focus-visible:outline-focus">
        <Scissors aria-hidden className="size-icon-xs" />
      </span>
    </button>
  ));
}
