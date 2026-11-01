// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";

import { type TakenOut } from "./taken-out";

// Everything taken out of the recording, drawn past the end of what plays
// where the recording would still run: one quiet band at the beats' height,
// as long as what was taken out, in two segments that each say how much
// they hold. What the squeeze took sits against the end of what plays,
// where its handle is, so squeezing visibly fills it; what the cuts deleted
// comes after, at the strip's end. Placed in px, so it glides with the
// strip's parts when the edit changes.

/** A hatch of thin diagonal lines in the fills' colour: what was squeezed. */
export const SQUEEZED_HATCH =
  "bg-[repeating-linear-gradient(135deg,var(--color-fill-strong)_0_1px,transparent_1px_6px)]";
/** A flat fill: what was deleted. */
const DELETED_FILL = "bg-fill";

function Segment({
  caption,
  className,
  from,
  px,
}: {
  /** What it says, or null with nothing in it. */
  caption: string | null;
  className: string;
  /** Where it starts, in px. */
  from: number;
  /** How wide it is, in px. */
  px: number;
}) {
  return (
    <span
      className="pointer-events-none absolute inset-y-0 overflow-hidden"
      style={{ insetInlineStart: from, width: Math.max(px, 0) }}
    >
      <span
        className={cn(
          "absolute inset-y-0 start-px end-px flex flex-wrap items-center justify-center overflow-hidden rounded-control border border-dashed border-fill-strong text-caption whitespace-nowrap text-fg-tertiary",
          className,
        )}
      >
        {/* As tall as the band, so the caption, where it does not fit
            beside it, wraps to a line the band hides rather than being cut
            short. */}
        <span className="h-full" />
        {caption === null ? null : <span className="px-1">{caption}</span>}
      </span>
    </span>
  );
}

export function TakenOutBand({
  from,
  pxPerSecond,
  taken,
}: {
  /** Where what plays ends, in px. */
  from: number;
  /** How wide a second is drawn, in px. */
  pxPerSecond: number;
  taken: TakenOut;
}) {
  const duration = useNumberFormatter({
    maximumFractionDigits: 1,
    style: "unit",
    unit: "second",
    unitDisplay: "short",
  });
  const squeezedPx = taken.squeezed * pxPerSecond;
  const deletedPx = taken.deleted * pxPerSecond;
  // Both are always there, so one grows from nothing and shrinks back to
  // it; empty, a segment says nothing.
  return (
    <>
      <Segment
        caption={
          squeezedPx < 1
            ? null
            : t("editor-timeline-squeezed", {
                duration: duration.format(taken.squeezed),
              })
        }
        className={SQUEEZED_HATCH}
        from={from}
        px={squeezedPx}
      />
      <Segment
        caption={
          deletedPx < 1
            ? null
            : t("editor-timeline-deleted", {
                duration: duration.format(taken.deleted),
              })
        }
        className={DELETED_FILL}
        from={from + squeezedPx}
        px={deletedPx}
      />
    </>
  );
}
