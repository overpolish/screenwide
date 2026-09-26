// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pin } from "lucide-react";
import { MouseEvent, PointerEvent } from "react";
import { Focusable } from "react-aria-components";

import { NativeTooltipTrigger } from "../../../components/shared/native-tooltip/native-tooltip-trigger";
import { RecordingAnnotationPin } from "../recording-annotations";
import {
  RecordingTimelineEdit,
  recordingTimelineSourceToOutput,
} from "../recording-timeline-edit";
import {
  RecordingPinStatus,
  RecordingPinStretch,
} from "../use-recording-pin-status";

import { SeekHandler } from "./timeline-seek";

/*
 * What a pinned clip shows of its pin inside its own block on the lane, in
 * three layers so the block's own controls stay in reach: the stretches sit
 * over the block's body and pass its presses on, the keyframes sit over the
 * stretches, and the trim handles the lane draws next sit over both.
 *
 * A clip in hand shows where it was pinned and put right by hand as
 * keyframes, which a right click or Delete takes away, the stretches the
 * tracker lost its content in, to check, and the stretches its content was
 * off the frame, where it is hidden. Any other pinned clip only says it is
 * pinned, so a lane of them stays readable. Every pinned clip shows how far
 * its path is worked out.
 */

type Block = {
  edit: RecordingTimelineEdit;
  /** The part of the clip this block shows, in output time from 0 to 1. */
  fragment: { outputEnd: number; outputStart: number };
  sourceDurationMs: number;
};

/** Where source times fall across one block of the lane. */
const blockGeometry = ({ edit, fragment, sourceDurationMs }: Block) => {
  const span = Math.max(fragment.outputEnd - fragment.outputStart, 1e-9);
  const output = (ms: number) =>
    recordingTimelineSourceToOutput(
      edit,
      Math.max(0, Math.min(1, ms / Math.max(sourceDurationMs, 1))),
    );
  const share = (ms: number) => (output(ms) - fragment.outputStart) / span;
  return {
    /** Where `ms` falls across the block, from 0 to 1, or null off it. */
    across: (ms: number) => {
      const at = share(ms);
      return at >= 0 && at <= 1 ? at : null;
    },
    output,
    /** The part of the block a stretch covers, or null where it misses. */
    stretch: ([start, end]: RecordingPinStretch) => {
      const from = Math.max(0, share(start));
      const to = Math.min(1, share(end));
      return to > from
        ? {
            left: `${String(from * 100)}%`,
            width: `${String((to - from) * 100)}%`,
          }
        : null;
    },
  };
};

/** The stretches the content was lost in and off the frame, each saying so
 * when the pointer rests on it. A press on one is a press on the block. */
export function RecordingAnnotationPinStretches({
  onBodyClick,
  onBodyPointerDown,
  status,
  ...block
}: Block & {
  onBodyClick: (event: MouseEvent) => void;
  onBodyPointerDown: (event: PointerEvent) => void;
  status?: RecordingPinStatus;
}) {
  if (!status) return null;
  const { stretch } = blockGeometry(block);
  const kinds = [
    {
      className: "bg-content/60",
      label: "Off the frame",
      ranges: status.hidden,
    },
    {
      className: "bg-warning/35",
      label: "Lost here, check",
      ranges: status.weak,
    },
  ];
  return kinds.flatMap(({ className, label, ranges }) =>
    ranges.map((range) => {
      const box = stretch(range);
      return box ? (
        // A grid, so the tooltip's own wrapper is stretched over the
        // stretch and the tooltip is placed against all of it.
        <div
          className="absolute inset-y-0 grid"
          key={`${label}:${String(range[0])}`}
          style={box}
        >
          <NativeTooltipTrigger tooltip={label}>
            <Focusable excludeFromTabOrder>
              <span
                aria-label={label}
                className={`block size-full ${className}`}
                onClick={onBodyClick}
                onPointerDown={onBodyPointerDown}
                role="img"
              />
            </Focusable>
          </NativeTooltipTrigger>
        </div>
      ) : null;
    }),
  );
}

/** The keyframes: where the clip was pinned and every correction. */
export function RecordingAnnotationPinKeyframes({
  label,
  onDeleteKeyframe,
  onKeyframeMenu,
  onSeek,
  pin,
  ...block
}: Block & {
  label: string;
  /** Take the keyframe at `ms` away. */
  onDeleteKeyframe: (ms: number) => void;
  /** Open the menu for the keyframe at `ms`, at the pointer. */
  onKeyframeMenu: (
    point: { x: number; y: number },
    ms: number,
    isPinnedFrame: boolean,
  ) => void;
  pin: RecordingAnnotationPin;
  onSeek?: SeekHandler;
}) {
  const { across, output } = blockGeometry(block);
  // The only keyframe is the pin itself: letting it go is Unpin's job, so it
  // offers nothing to delete.
  const deletable = pin.keyframes.length > 1;
  return pin.keyframes.map((keyframe) => {
    const share = across(keyframe.ms);
    if (share === null) return null;
    const isPin = keyframe.ms === pin.pinnedMs;
    return (
      // The hit area is wider than the diamond it holds, so a keyframe can be
      // taken hold of without aiming at eight pixels.
      <button
        aria-label={`${isPin ? "Pinned frame" : "Correction"} of ${label}`}
        className="group absolute inset-y-0 w-4 -translate-x-1/2 focus-visible:outline-none"
        key={keyframe.ms}
        onClick={(event) => {
          event.stopPropagation();
          onSeek?.(output(keyframe.ms), "end");
        }}
        onContextMenu={(event) => {
          event.preventDefault();
          event.stopPropagation();
          if (deletable)
            onKeyframeMenu(
              { x: event.clientX, y: event.clientY },
              keyframe.ms,
              isPin,
            );
        }}
        onKeyDown={(event) => {
          if (event.key !== "Delete" && event.key !== "Backspace") return;
          event.preventDefault();
          event.stopPropagation();
          if (deletable) onDeleteKeyframe(keyframe.ms);
        }}
        onPointerDown={(event) => {
          event.stopPropagation();
        }}
        style={{ left: `${String(share * 100)}%` }}
        type="button"
      >
        <span
          aria-hidden="true"
          className="absolute top-1/2 left-1/2 size-2 -translate-x-1/2 -translate-y-1/2 rotate-45 rounded-[1px] bg-primary-fg group-focus-visible:outline-2 group-focus-visible:outline-offset-1 group-focus-visible:outline-primary-fg"
        />
      </button>
    );
  });
}

/** What every pinned clip shows: that it is pinned, and how far its path is
 * worked out. Neither takes the pointer, so trimming and dragging reach the
 * block under them; pinning itself is in the clip's menu. */
export function RecordingAnnotationPinBadge({
  label,
  selected,
  status,
}: {
  label: string;
  selected: boolean;
  status?: RecordingPinStatus;
}) {
  const progress = status?.progress ?? null;
  return (
    <>
      <Pin
        aria-hidden="true"
        className={`pointer-events-none absolute top-1/2 right-control-inset size-icon-mini -translate-y-1/2 ${selected ? "text-primary-fg" : "text-content-fg-secondary"}`}
      />
      {progress !== null ? (
        <div
          aria-label={`Tracking ${label}`}
          aria-valuemax={100}
          aria-valuemin={0}
          aria-valuenow={Math.round(progress * 100)}
          className={`pointer-events-none absolute bottom-0 left-0 h-0.5 ${selected ? "bg-primary-fg" : "bg-primary"}`}
          role="progressbar"
          style={{ width: `${String(progress * 100)}%` }}
        />
      ) : null}
    </>
  );
}
