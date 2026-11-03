// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowRightLeft } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { TimelineMark } from "../timeline-mark";
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
    <TimelineMark
      end="bottom"
      glyph={<ArrowRightLeft aria-hidden />}
      key={jump}
      label={t("editor-timeline-put-back")}
      onPress={() => {
        onPutBack(jump);
      }}
      style={at(scale, jump)}
    />
  ));
}
