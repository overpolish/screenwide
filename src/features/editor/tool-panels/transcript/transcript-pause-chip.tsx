// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioLinesOff } from "lucide-react";
import { useNumberFormatter } from "react-aria";

/** What a pause badge says: its glyph and how long the silence runs. Inline
 * rather than a flex row, so the badge still sits in the line and wraps like
 * a word. */
export function TranscriptPauseChip({ durationMs }: { durationMs: number }) {
  const seconds = useNumberFormatter({
    maximumFractionDigits: 1,
    style: "unit",
    unit: "second",
    unitDisplay: "short",
  });
  return (
    <>
      <AudioLinesOff
        aria-hidden
        className="me-inline inline size-icon-xs align-[-0.1em]"
      />
      {seconds.format(durationMs / 1_000)}
    </>
  );
}
