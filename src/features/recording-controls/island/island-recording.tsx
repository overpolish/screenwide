// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic, MicOff } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { type CSSProperties, type ReactNode } from "react";
import { useMeter } from "react-aria";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { IslandRow } from "../../../primitives/island/island-row";
import { formatBytes } from "../../editor/duration";

import { LevelFill } from "./island-glyphs";
import { BreathingDot, LiveControls } from "./island-live";
import { clock, INPUT_GLYPHS } from "./island-names";

import type { IslandMoment, RecordingIslandProps } from "./island-types";

// The island while it records, all on its one row: the recording dot and,
// with confidence checks on, what is being recorded from; then the size so
// far, the time, and under the pointer the controls, the row growing
// sideways to take them rather than hanging them beneath. A moment placed by
// its shortcut shows in the time's place for as long as it is shown.

const swap = {
  duration: motionDurations.travel,
  ease: motionEasings.out,
} as const;

/** A moment in the time's place: its kind's dot and name; held long enough,
 * the voice note it is taking, by the kind's colour and the note's time; or
 * that there is no microphone to take one. */
function MomentShown({ moment }: { moment: IslandMoment }) {
  // The kind's own colour, drawn whole rather than at the text's alpha.
  const tint = { "--icon-alpha": 1, color: moment.color } as CSSProperties;
  if (moment.noteMs !== null)
    return (
      <span
        aria-label={t("recording-controls-recording-note", {
          moment: moment.name,
        })}
        className="flex items-center gap-inline px-2 text-label tabular-nums"
        role="status"
      >
        <span className="flex" style={tint}>
          <Mic aria-hidden className="size-icon" />
        </span>
        {clock(moment.noteMs / 1_000)}
      </span>
    );
  if (moment.noMicrophone)
    return (
      <span
        aria-label={t("recording-controls-note-no-microphone")}
        className="flex items-center gap-inline px-2 text-label"
        role="status"
      >
        <span className="flex text-fg-secondary">
          <MicOff aria-hidden className="size-icon" />
        </span>
        {t("recording-controls-note-no-microphone-short")}
      </span>
    );
  return (
    <span
      aria-label={t("recording-controls-moment-placed", {
        moment: moment.name,
      })}
      className="flex items-center gap-inline px-2 text-label"
      role="status"
    >
      <span
        className="size-2 shrink-0 rounded-full"
        style={{ backgroundColor: moment.color }}
      />
      <span className="max-w-32 truncate" dir="auto">
        {moment.name}
      </span>
    </span>
  );
}

/** A recorded input's level, as its glyph filling as the resting row's do. */
function ConfidenceMeter({
  kind,
  level = 0,
}: {
  kind: "microphone" | "systemAudio";
  level?: number;
}) {
  const { meterProps } = useMeter({
    "aria-label":
      kind === "microphone"
        ? t("recording-controls-microphone-level")
        : t("recording-controls-system-audio-level"),
    value: level * 100,
  });
  return (
    <span
      {...meterProps}
      className="relative flex size-island-control shrink-0"
    >
      <LevelFill icon={INPUT_GLYPHS[kind].on} level={level} />
    </span>
  );
}

/** The camera being recorded, small and round. Its picture fills its whole
 * control where the glyphs leave room round theirs, so straight after the
 * dot it keeps the distance a glyph's room gives. */
function CameraConfidence({
  followsDot,
  preview,
}: {
  followsDot: boolean;
  preview: ReactNode;
}) {
  return (
    <span
      aria-label={t("recording-controls-camera-preview")}
      className={cn(
        "flex size-island-control shrink-0 overflow-hidden rounded-full bg-fill",
        followsDot && "ms-2",
      )}
      role="img"
    >
      {preview}
    </span>
  );
}

/** How much is on disk so far: a spinner until it is first measured, then
 * the size, updating in place so it never pulls the eye from the time. It
 * starts in megabytes, as kilobytes would change unit within seconds. */
function RecordedSize({ bytes }: { bytes: number | null }) {
  return (
    <span className="flex text-label whitespace-nowrap text-fg-tertiary tabular-nums">
      {bytes !== null && bytes > 0 ? (
        formatBytes(bytes, { decimals: 1, smallestUnit: "megabyte" })
      ) : (
        <CircularProgress
          aria-label={t("recording-controls-measuring-size")}
          isIndeterminate
          size="small"
        />
      )}
    </span>
  );
}

export function RecordingRow({
  props,
  showsControls,
}: {
  props: RecordingIslandProps;
  showsControls: boolean;
}) {
  const isPaused = props.phase === "paused";
  const hasNotch = Boolean(props.notch);
  // Around a notch the size keeps the dot company, leaving the time alone
  // beside the notch; without one it sits before the time.
  const size =
    props.writtenBytes === undefined ? null : (
      <RecordedSize bytes={props.writtenBytes} />
    );
  return (
    <IslandRow
      end={
        <>
          {hasNotch ? null : size}
          {/* The time, or a moment in its place, crossfading as one gives
              way to the other. */}
          <AnimatePresence initial={false} mode="popLayout">
            <motion.span
              animate={{ filter: "blur(0px)", opacity: 1 }}
              className="flex"
              exit={{ filter: "blur(4px)", opacity: 0 }}
              initial={{ filter: "blur(4px)", opacity: 0 }}
              key={
                props.moment
                  ? `moment ${String(props.moment.key)} ${props.moment.noteMs !== null ? "note" : props.moment.noMicrophone ? "none" : "kind"}`
                  : "time"
              }
              transition={swap}
            >
              {props.moment ? (
                <MomentShown moment={props.moment} />
              ) : (
                <span
                  aria-label={t(
                    isPaused
                      ? "recording-controls-island-paused"
                      : "recording-controls-island-elapsed",
                    { time: clock(props.elapsed ?? 0) },
                  )}
                  className={cn(
                    "px-2 text-label tabular-nums",
                    isPaused && "text-fg-secondary",
                  )}
                  role="timer"
                >
                  {clock(props.elapsed ?? 0)}
                </span>
              )}
            </motion.span>
          </AnimatePresence>
          {/* Its own presence, so the controls come into focus as the row
              grows for them, and are lifted out to fade as it shrinks,
              rather than appearing and going at once. */}
          <AnimatePresence initial={false} mode="popLayout">
            {showsControls ? (
              <motion.span
                animate={{ filter: "blur(0px)", opacity: 1, scale: 1 }}
                className="flex"
                exit={{ filter: "blur(4px)", opacity: 0, scale: 0.96 }}
                initial={{ filter: "blur(4px)", opacity: 0, scale: 0.96 }}
                key="controls"
                transition={swap}
              >
                <LiveControls {...props} isPaused={isPaused} />
              </motion.span>
            ) : null}
          </AnimatePresence>
        </>
      }
      start={
        <span className="flex items-center gap-1 ps-2">
          <BreathingDot isPaused={isPaused} />
          {hasNotch && size ? <span className="ms-1 flex">{size}</span> : null}
          {props.hasConfidenceChecks ? (
            <>
              {props.microphone.isOn ? (
                <ConfidenceMeter
                  kind="microphone"
                  level={isPaused ? 0 : props.microphone.level}
                />
              ) : null}
              {props.camera.isOn ? (
                <CameraConfidence
                  followsDot={!props.microphone.isOn && !(hasNotch && size)}
                  preview={props.cameraPreview}
                />
              ) : null}
              {props.systemAudio.isOn ? (
                <ConfidenceMeter
                  kind="systemAudio"
                  level={isPaused ? 0 : props.systemAudio.level}
                />
              ) : null}
            </>
          ) : null}
        </span>
      }
    />
  );
}
