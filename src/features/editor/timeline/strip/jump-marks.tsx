// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowRightLeft } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { at, type TimelineScale } from "../timeline-scale";

// Once something is moved, a mark wherever the recording no longer plays in
// the order it was recorded. A cut's seam marks the top of the beats; a
// jump marks their foot, so the two read apart where they meet.

/** A mark where play jumps, at the recorded moment `jumps` names: pressed,
 * it puts what was moved there back where it was recorded. */
export function JumpMarks({
  jumps,
  onPutBack,
  scale,
}: {
  jumps: number[];
  onPutBack: (at: number) => void;
  scale: TimelineScale;
}) {
  return jumps.map((jump) => (
    <button
      aria-label={t("editor-timeline-put-back")}
      className="group absolute inset-y-0 z-10 flex w-4 -translate-x-1/2 cursor-default items-end justify-center outline-none"
      dir="ltr"
      key={jump}
      onClick={() => {
        onPutBack(jump);
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={at(scale, jump)}
      type="button"
    >
      <span className="absolute inset-y-0 w-px bg-fg-secondary" />
      <span className="relative -mb-1 flex size-4 items-center justify-center rounded-full bg-fg text-surface group-focus-visible:outline-2 group-focus-visible:outline-offset-1 group-focus-visible:outline-focus">
        <ArrowRightLeft aria-hidden className="size-icon-xs" />
      </span>
    </button>
  ));
}
