// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FastForward, Hourglass, MonitorPlay, Pin, Snail } from "lucide-react";
import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { placed, type Span, type TimelineScale } from "../timeline-scale";

import { SpringEnd } from "./spring-end";
import { SpringWire } from "./spring-wire";

import type { LullPlay } from "../../island/lull/lull-island";

// A lull between beats, drawn as its spring, which only ever stands for
// speed. Pressed, it is chosen; double-clicked, it is kept as recorded, out
// of the squeeze, or given back to it. Chosen, its end retimes it, or with
// ⌥ held, trims it. Its badge says how it plays: kept, faster or slower
// than recorded, or dead air still to be dealt with. A lull plays at one
// speed throughout; where the picture moves by itself in it, a screen
// playing sits on the coil there, since that is what holds the speed down.

/** The narrowest a lull shows its badge at, in px, unless chosen. */
const BADGE_PX = 56;
/** How near its recorded length, in px, a dragged end snaps to keeping it. */
const KEEP_SNAP_PX = 8;
/** The least width, in px, a lull can be pressed at. */
const HIT_PX = 14;

export type SpringProps = {
  /** Whether ⌥ is held, which turns the chosen end into a trim. */
  isAltHeld: boolean;
  /** Whether the lull is long enough still to be dead air. */
  isDeadAir: boolean;
  isSelected: boolean;
  onPlayChange: (play: LullPlay) => void;
  onPress: () => void;
  /** Told the recorded moment the lull now ends at, trimmed. */
  onTrim: (end: number) => void;
  play: LullPlay;
  /** How long the lull plays now, in seconds. */
  plays: number;
  scale: TimelineScale;
  span: Span;
  /** The strip's width, in px. */
  width: number;
  /** Where the picture moves by itself in the lull, such as a page loading,
   * which holds down how fast the whole lull can play. */
  moving?: Span[];
};

export function Spring({
  isAltHeld,
  isDeadAir,
  isSelected,
  moving = [],
  onPlayChange,
  onPress,
  onTrim,
  play,
  plays,
  scale,
  span,
  width,
}: SpringProps) {
  const seconds = useNumberFormatter({
    maximumFractionDigits: 1,
    style: "unit",
    unit: "second",
    unitDisplay: "short",
  });
  const recorded = span.end - span.start;
  const startX = scale.toX(span.start);
  const spanX = scale.toX(span.end) - startX;
  const px = Math.max(0, spanX * width);
  // Squeezed narrower than a fingertip, it reaches over the edges of the
  // beats beside it, and sits above them there, so it can still be pressed.
  const isNarrow = px < HIT_PX;
  const isSet = play.kind !== "squeeze";
  const look = play.kind === "keep" ? "kept" : isDeadAir ? "dead-air" : "plain";
  const speed = speedOf(recorded, plays);
  const hasBadge = isSet || isDeadAir || speed !== null;
  const isBadgeShown = px > BADGE_PX || isSelected;
  // The outline: chosen, or dead air still to be dealt with. Unchosen and
  // narrow, it keeps to the drawn width, off the beats beside it.
  const outline = isSelected
    ? "-inset-0.5 border-2 border-primary"
    : isDeadAir && !isSet
      ? "inset-y-0 border border-dashed border-fg-tertiary"
      : null;
  const outlineInset = isSelected || !isNarrow ? 0 : (HIT_PX - px) / 2;

  return (
    <button
      aria-label={t("editor-timeline-lull", {
        duration: seconds.format(recorded),
      })}
      aria-pressed={isSelected}
      className={cn(
        "group absolute inset-y-1.5 cursor-default rounded-part outline-none transition-colors focus-visible:outline-2 focus-visible:outline-focus",
        // Chosen, it sits over the beats beside it, so its end handle,
        // which reaches past its edge, can be taken hold of.
        isSelected
          ? "z-10 bg-selected"
          : cn("hover:bg-fill", isNarrow && "z-[5]"),
      )}
      onClick={onPress}
      onDoubleClick={() => {
        onPlayChange({ kind: play.kind === "keep" ? "squeeze" : "keep" });
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={
        isNarrow
          ? {
              insetInlineStart: startX * width - (HIT_PX - px) / 2,
              width: HIT_PX,
            }
          : placed(scale, span.start, span.end)
      }
      type="button"
    >
      {outline ? (
        <span
          aria-hidden
          className={cn("pointer-events-none absolute rounded-part", outline)}
          style={isSelected ? undefined : { insetInline: outlineInset }}
        />
      ) : null}
      <SpringWire
        gaps={
          isNarrow
            ? []
            : moving.map(
                (stretch) =>
                  (scale.toX((stretch.start + stretch.end) / 2) - startX) *
                  width,
              )
        }
        look={look}
        seconds={recorded}
        width={px}
      />
      {isNarrow
        ? null
        : moving.map((stretch) => (
            // In the middle of the stretch, on the coil's line, however
            // short the stretch is, where the wire fades out round it.
            <MonitorPlay
              aria-hidden
              className="pointer-events-none absolute top-1/2 size-icon-xs -translate-1/2 text-fg-secondary"
              key={stretch.start}
              style={{
                insetInlineStart: `${String(((scale.toX((stretch.start + stretch.end) / 2) - startX) / spanX) * 100)}%`,
              }}
            />
          ))}
      {hasBadge ? (
        // Over the box, never across its outline. Squeezed too narrow to
        // carry it, the badge waits for the pointer or the selection, since
        // the lulls squeezed hardest are the ones whose speed matters most.
        <span
          aria-hidden
          className={cn(
            "pointer-events-none absolute inset-x-0 bottom-full z-10 mb-1 flex justify-center transition-opacity",
            !isBadgeShown && "opacity-0 group-hover:opacity-100",
          )}
        >
          <Badge>
            <PlayGlyph play={play} speed={speed} />
            {speed === null ? null : <SpeedLabel speed={speed} />}
          </Badge>
        </span>
      ) : null}
      {isSelected ? (
        <SpringEnd
          isAltHeld={isAltHeld}
          onDrag={(share, isTrim) => {
            // What is drawn up to the pointer is kept, played as recorded,
            // so a retimed lull's end stays under the pointer as it goes
            // back to real speed.
            if (isTrim) {
              onTrim(span.start + plays * share);
              return;
            }
            const to = plays * share;
            // Drawn to within a few px of its recorded length, it is the
            // recorded lull again: kept, not retimed.
            const off = (Math.abs(recorded - to) / plays) * px;
            onPlayChange(
              off < KEEP_SNAP_PX || to <= 0
                ? { kind: "keep" }
                : { kind: "retimed", speed: recorded / to },
            );
          }}
        />
      ) : null}
    </button>
  );
}

function PlayGlyph({ play, speed }: { play: LullPlay; speed: number | null }) {
  if (play.kind === "keep") return <Pin aria-hidden />;
  if (speed === null) return <Hourglass aria-hidden />;
  return speed > 1 ? <FastForward aria-hidden /> : <Snail aria-hidden />;
}

function SpeedLabel({ speed }: { speed: number }) {
  const digits = speed < 10 ? 1 : 0;
  const rate = useNumberFormatter({
    maximumFractionDigits: digits,
    minimumFractionDigits: digits,
  });
  return t("editor-timeline-rate", { rate: rate.format(speed) });
}

/** How fast a lull plays against real time, as a multiple, or null where it
 * plays as recorded. A length in seconds says nothing to someone judging by
 * watching; how much it was hurried or slowed does. */
const speedOf = (recorded: number, plays: number) => {
  if (plays <= 0) return null;
  const times = recorded / plays;
  return Math.abs(times - 1) < 0.05 ? null : times;
};
