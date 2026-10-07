// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ContentRotate } from "../../../components/base/content-rotate/content-rotate";
import { cn } from "../../../lib/styling";

import type { PlacedMoment } from "./use-placed-moment";

/**
 * Rotates each digit on its own, so a tick only animates what actually
 * changed: 58 to 59 moves the units alone, while 59 to 00 moves both. Rotating
 * the pair as one unit would swing the tens digit on every single second.
 */
function RotatingDigits({ value }: { value: string }) {
  const leading = value.slice(0, -1);
  const last = value.slice(-1);

  return (
    <>
      <ContentRotate contentKey={leading}>{leading}</ContentRotate>
      <ContentRotate contentKey={last}>{last}</ContentRotate>
    </>
  );
}

/** The timer, which a placed moment's kind replaces for a moment, in the
 * slot the timer already holds so the pill never changes width. */
export function RecordingDockElapsed({
  hours,
  isPaused,
  minutes,
  moment,
  seconds,
}: {
  hours: string;
  isPaused: boolean;
  minutes: string;
  moment: PlacedMoment | null;
  seconds: string;
}) {
  return (
    <ContentRotate
      containerClassName="w-full"
      contentKey={moment ? `moment-${moment.key.toString()}` : "timer"}
    >
      {moment ? (
        <div
          aria-label={`${moment.name} moment placed`}
          className="flex min-w-0 items-center justify-center gap-control"
          role="status"
        >
          <span
            className="size-control-inset shrink-0 rounded-full"
            style={{ backgroundColor: moment.color }}
          />
          <span className="truncate">{moment.name}</span>
        </div>
      ) : (
        <div
          className={cn(
            "flex justify-center transition-colors",
            isPaused && "text-content-fg-secondary",
          )}
        >
          <RotatingDigits value={hours} />:
          <RotatingDigits value={minutes} />:
          <RotatingDigits value={seconds} />
        </div>
      )}
    </ContentRotate>
  );
}
