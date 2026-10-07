// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic, MicOff } from "lucide-react";

import { ContentRotate } from "../../../components/base/content-rotate/content-rotate";
import { cn } from "../../../lib/styling";

import type { PlacedMoment } from "./use-placed-moment";

/** A note's length as minutes and seconds, `0:04`. */
const noteTime = (ms: number) => {
  const seconds = Math.floor(ms / 1_000);
  return `${Math.floor(seconds / 60).toString()}:${(seconds % 60).toString().padStart(2, "0")}`;
};

/** What a placed moment shows: its kind, then, held long enough, the note it
 * is recording, or that there is no microphone to record one. */
function MomentShown({ moment }: { moment: PlacedMoment }) {
  if (moment.noteMs !== null)
    return (
      <div
        aria-label={`Recording a ${moment.name} voice note`}
        className="flex min-w-0 items-center justify-center gap-control"
        role="status"
      >
        <Mic
          aria-hidden="true"
          className="size-icon-small shrink-0"
          style={{ color: moment.color }}
        />
        <span>{noteTime(moment.noteMs)}</span>
      </div>
    );
  if (moment.noMicrophone)
    return (
      <div
        aria-label="No microphone for a voice note"
        className="flex min-w-0 items-center justify-center gap-control"
        role="status"
      >
        <MicOff aria-hidden="true" className="size-icon-small shrink-0" />
        <span className="truncate">No mic</span>
      </div>
    );
  return (
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
  );
}

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
      contentKey={
        moment
          ? `moment-${moment.key.toString()}-${moment.noteMs !== null ? "note" : moment.noMicrophone ? "none" : "kind"}`
          : "timer"
      }
    >
      {moment ? (
        <MomentShown moment={moment} />
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
