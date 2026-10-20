// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowUpRight, EyeOff, MapPinPlusInside, Type } from "lucide-react";

import { cn } from "../../../../lib/styling";

import {
  ANNOTATIONS,
  CAPTIONS,
  SCENES,
  SHORTCUTS,
  span,
  STRETCHES,
} from "./combined-data";
import { Band, Sound, TimelineFrame } from "./timeline-parts";
import { Filmstrip } from "./timeline-stills";

// Final Cut's magnetic timeline, for a recording: no lanes and no names
// column. The recording is the one storyline, and what belongs to it lives
// inside it, each where it fits: the scene along the top, the shortcuts on
// the picture they appear over, the captions under the sound they come
// from. Only annotations, which are placed by hand, ride above as connected
// clips, joined to the storyline by a stem so moving the footage carries
// them with it. They stack only where they overlap.

const ANNOTATION_ICONS: Record<string, typeof Type> = {
  Arrow: ArrowUpRight,
  Counter: MapPinPlusInside,
  Redaction: EyeOff,
  Text: Type,
};

/** Each clip goes in the lowest row it fits, so the stack is only as tall
 * as the overlaps at the busiest moment. */
function pack<T extends { end: number; start: number }>(items: T[]) {
  const ends: number[] = [];
  return [...items]
    .sort((first, second) => first.start - second.start)
    .map((item) => {
      let row = ends.findIndex((end) => end <= item.start);
      if (row < 0) row = ends.length;
      ends[row] = item.end;
      return { ...item, row };
    });
}

const ROW = 18;
const GAP = 2;

function ConnectedAnnotations() {
  const placed = pack(ANNOTATIONS);
  const rows = Math.max(...placed.map(({ row }) => row)) + 1;
  return (
    <div className="relative" dir="ltr" style={{ height: rows * (ROW + GAP) }}>
      {placed.map(({ end, isSelected, label, row, start }) => {
        const bottom = row * (ROW + GAP) + GAP;
        const Icon = ANNOTATION_ICONS[label.split(" ")[0] ?? ""] ?? Type;
        return (
          <span key={start}>
            {/* The stem joining the clip to its moment on the storyline. */}
            <span
              className={cn(
                "absolute bottom-0 w-px",
                isSelected ? "bg-(--accent)" : "bg-(--fg-3)/40",
              )}
              style={{
                height: bottom,
                insetInlineStart: span(start, end).insetInlineStart,
              }}
            />
            {/* The icon says what it is; the name is for assistive
                technology and the tooltip, not for reading at a glance. */}
            <span
              aria-label={label}
              className={cn(
                "absolute flex items-center overflow-hidden rounded-[5px] px-1",
                isSelected
                  ? "combined-accent"
                  : "bg-(--clip-annotation) text-(--fg)",
              )}
              style={{ ...span(start, end), bottom, height: ROW }}
              title={label}
            >
              <Icon className="size-3 shrink-0" />
            </span>
          </span>
        );
      })}
    </div>
  );
}

/** Text placed along a band of the storyline, each piece over its time. */
function Labels({
  className,
  items,
}: {
  className: string;
  items: { end: number; label: string; start: number }[];
}) {
  return items.map(({ end, label, start }) => (
    <span
      className={cn("absolute inset-y-0 truncate", className)}
      key={start}
      style={span(start, end)}
    >
      {label}
    </span>
  ));
}

/** The two voices half and half, as the camera and screen share the
 * picture: system audio above, the microphone below. */
function Voices() {
  return (
    <>
      <Band className="top-0 h-1/2">
        <Sound className="text-(--fg-3)" seed={1} />
      </Band>
      <Band className="bottom-0 h-1/2">
        <Sound className="text-(--level)" seed={4} />
      </Band>
    </>
  );
}

/** The recording, top to bottom: the scene in force, the camera, the
 * screen with the shortcuts shown over it, the sound, and the captions
 * spoken in it. */
function Storyline({ hasPicture }: { hasPicture: boolean }) {
  return (
    <div className={cn("relative", hasPicture ? "h-20" : "h-8")} dir="ltr">
      {STRETCHES.map(([start, end]) => (
        <span
          className="absolute inset-y-0 overflow-hidden rounded-[6px] bg-(--fill-2) shadow-[inset_0_0_0_1px_var(--divider)]"
          key={start}
          style={span(start, end)}
        />
      ))}
      {hasPicture ? (
        <>
          <Band className="top-0 h-3">
            <Labels
              className="border-s-2 border-(--clip-scene) ps-1 text-[9px] leading-3 font-medium text-(--fg-2)"
              items={SCENES}
            />
          </Band>
          <Band className="top-3 h-3">
            <Filmstrip source="camera" />
          </Band>
          <Band className="top-6 h-6">
            <Filmstrip source="screen" />
          </Band>
          {/* Each shortcut over the picture for as long as it shows: a
              tint across its time, its keycap at the start. */}
          <Band className="top-6 h-6">
            {SHORTCUTS.map(({ end, label, start }) => (
              <span
                className="absolute inset-y-0.5 flex items-center rounded-[4px] bg-black/35 ps-0.5 ring-1 ring-white/40"
                key={start}
                style={span(start, end)}
              >
                <span className="combined-raised rounded-[3px] px-1 text-[9px] leading-4 font-medium whitespace-nowrap text-(--fg-2)">
                  {label}
                </span>
              </span>
            ))}
          </Band>
          <Band className="top-12 h-4">
            <Voices />
          </Band>
          <Band className="bottom-0 h-3">
            <Labels
              className="border-s-2 border-(--fg-3)/50 ps-1 text-[9px] leading-3 text-(--fg-3)"
              items={CAPTIONS}
            />
          </Band>
        </>
      ) : (
        <Band className="inset-y-0">
          <Voices />
        </Band>
      )}
    </div>
  );
}

export function MagneticTimeline({ hasPicture }: { hasPicture: boolean }) {
  return (
    <TimelineFrame labelWidth={0}>
      {hasPicture ? <ConnectedAnnotations /> : null}
      <Storyline hasPicture={hasPicture} />
    </TimelineFrame>
  );
}
