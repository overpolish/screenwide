// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic, MicOff } from "lucide-react";

import { ContentRotate } from "../../../components/base/content-rotate/content-rotate";
import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { AudioMeter } from "../../audio-inputs/audio-meter";

import type { PlacedMoment } from "./use-placed-moment";

/** A note's length as minutes and seconds, `0:04`. */
const noteTime = (ms: number) => {
  const seconds = Math.floor(ms / 1_000);
  return `${Math.floor(seconds / 60).toString()}:${(seconds % 60).toString().padStart(2, "0")}`;
};

/** What a placed moment shows: its kind, then, held long enough, the note it
 * is recording, or that there is no microphone to record one. A note's level
 * shows beside it when the dock's confidence checks are on. */
function MomentShown({
  moment,
  noteDecibels,
}: {
  moment: PlacedMoment;
  noteDecibels: number | undefined;
}) {
  if (moment.noteMs !== null)
    return (
      <div
        aria-label={t("recording-controls-recording-note", {
          moment: moment.name,
        })}
        className="flex min-w-0 items-center justify-center gap-control"
        role="status"
      >
        {noteDecibels !== undefined && (
          <AudioMeter
            decibels={noteDecibels}
            height={16}
            hidePeakTick
            hideTicks
            orientation="vertical"
            radius={1}
            width={2}
          />
        )}
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
        aria-label={t("recording-controls-note-no-microphone")}
        className="flex min-w-0 items-center justify-center gap-control"
        role="status"
      >
        <MicOff aria-hidden="true" className="size-icon-small shrink-0" />
        <span className="truncate">
          {t("recording-controls-note-no-microphone-short")}
        </span>
      </div>
    );
  return (
    <div
      aria-label={t("recording-controls-moment-placed", {
        moment: moment.name,
      })}
      className="flex min-w-0 items-center justify-center gap-control"
      role="status"
    >
      <span
        className="size-control-inset shrink-0 rounded-full"
        style={{ backgroundColor: moment.color }}
      />
      <span className="truncate" dir="auto">
        {moment.name}
      </span>
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
  noteDecibels,
  seconds,
}: {
  hours: string;
  isPaused: boolean;
  minutes: string;
  moment: PlacedMoment | null;
  /** The note's level, or undefined when confidence checks are off. */
  noteDecibels: number | undefined;
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
        <MomentShown moment={moment} noteDecibels={noteDecibels} />
      ) : (
        <div
          className={cn(
            "flex justify-center transition-colors",
            isPaused && "text-content-fg-secondary",
          )}
          // A clock reads left to right in every language; its digits are
          // laid out one by one, so they would otherwise run backwards.
          dir="ltr"
        >
          <RotatingDigits value={hours} />:
          <RotatingDigits value={minutes} />:
          <RotatingDigits value={seconds} />
        </div>
      )}
    </ContentRotate>
  );
}
