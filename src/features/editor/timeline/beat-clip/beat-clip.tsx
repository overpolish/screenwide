// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isLight } from "../../../../composites/color-grid/palette";
import { cn } from "../../../../lib/styling";
import { BEAT_GLYPHS } from "../../glyphs/beat-glyphs";
import { Waveform } from "../../waveform/waveform";
import { reach, type Span, type TimelineScale } from "../timeline-scale";

import { beatLabel, type TimelineBeat } from "./beat-label";

// A beat: something that holds its time, done, said, flagged, loud or newly
// shown. Done or newly shown, it is drawn as the screen at that moment with
// what happened on it, so a recording with no voice is read by its
// pictures; flagged, the same, its flag in its kind's colour, so moments
// read apart from what was merely done; said or loud, as its sound, with a
// line's words over it. What it shows grows with its width: bare, then its
// glyph, its picture, and last its title.

/** The narrowest a beat carries its glyph at, in px: the glyph and padding. */
const GLYPH_PX = 18;
/** The narrowest a beat shows its picture beside its glyph at, in px. */
const THUMBNAIL_PX = 64;
/** The narrowest a beat shows its title at, in px. */
const TITLE_PX = 170;

export type BeatClipProps = {
  beat: TimelineBeat;
  isSelected: boolean;
  onPress: () => void;
  /** The recording's shape, width over height, which its pictures keep.
   * Known before any picture is read, so a picture's place is held at its
   * size while it loads and nothing moves when it arrives. */
  pictureAspect: number;
  scale: TimelineScale;
  span: Span;
  /** The strip's width, in px. */
  width: number;
  /** Whether what it draws over the picture, such as a shortcut's keys, is
   * hidden: it is drawn as an outline, still there to press. */
  isHidden?: boolean;
  /** The beat's sound, as peaks from 0 to 1: a line's voice, or the noise
   * that made a moment loud. */
  peaks?: number[];
  /** The screen at the beat, or null while it is read from the video. */
  thumbnail?: string | null;
};

export function BeatClip({
  beat,
  isHidden = false,
  isSelected,
  onPress,
  peaks,
  pictureAspect,
  scale,
  span,
  thumbnail = null,
  width,
}: BeatClipProps) {
  const { pad, px, style } = reach(scale, span, width);
  if (px < 1) return null;
  const label = beatLabel(beat);
  const hasGlyph = px >= GLYPH_PX;
  // Too narrow for its flag, a moment is its kind's colour, so it still
  // reads as one.
  const bareColor = beat.kind === "moment" && !hasGlyph ? beat.color : null;
  return (
    // The button reaches past a narrow block, so a beat a few pixels wide is
    // still easy to press; the block is drawn at its own width inside it.
    <button
      aria-label={label}
      aria-pressed={isSelected}
      className="group absolute inset-y-0 cursor-default outline-none"
      onClick={onPress}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={style}
      type="button"
    >
      <span
        className={cn(
          "absolute inset-y-0 flex items-center gap-1.5 overflow-hidden rounded-control text-start text-caption whitespace-nowrap transition-colors group-focus-visible:outline-2 group-focus-visible:outline-offset-1 group-focus-visible:outline-focus",
          // Padding would hold a narrow block wider than its time, over
          // its neighbours; too narrow for its glyph, it is drawn bare.
          hasGlyph && "px-1",
          isSelected
            ? "bg-primary text-on-primary"
            : isHidden
              ? "border border-dashed border-fg-tertiary text-fg-tertiary group-hover:bg-fill"
              : "bg-fill-strong text-fg-secondary group-hover:bg-fg-tertiary group-hover:text-fg",
        )}
        style={{
          insetInlineEnd: pad,
          insetInlineStart: pad,
          ...(bareColor && !isSelected ? { backgroundColor: bareColor } : null),
        }}
      >
        {beat.kind === "speech" || beat.kind === "sound" ? (
          <>
            {peaks ? <BeatSound label={label} peaks={peaks} px={px} /> : null}
            {beat.kind === "speech" ? (
              <span className="relative mb-auto truncate pt-1" dir="auto">
                {beat.line}
              </span>
            ) : (
              <span className="relative mb-auto flex items-center gap-1.5 pt-1">
                {hasGlyph ? <BeatGlyph beat={beat} /> : null}
                {px > TITLE_PX ? (
                  <span className="truncate">{label}</span>
                ) : null}
              </span>
            )}
          </>
        ) : (
          <>
            {px > THUMBNAIL_PX ? (
              <Picture aspect={pictureAspect} thumbnail={thumbnail} />
            ) : null}
            {hasGlyph ? <BeatGlyph beat={beat} /> : null}
            {px > TITLE_PX ? (
              <span className="truncate" dir="auto">
                {/* The flag already says it is a moment; its name says
                    which kind. Assistive technology hears "… moment". */}
                {beat.kind === "moment" ? beat.name : label}
              </span>
            ) : null}
          </>
        )}
      </span>
    </button>
  );
}

/** The screen at the beat, at the recording's own shape and a set height;
 * while it is read from the video, a pulsing fill of the same size. */
function Picture({
  aspect,
  thumbnail,
}: {
  aspect: number;
  thumbnail: string | null;
}) {
  const className = "h-9 max-w-16 shrink-0 rounded-sm";
  const style = { aspectRatio: aspect };
  return thumbnail ? (
    <img
      alt=""
      className={cn(className, "object-cover")}
      draggable={false}
      src={thumbnail}
      style={style}
    />
  ) : (
    <span
      aria-hidden
      className={cn(
        className,
        "animate-pulse bg-fill motion-reduce:animate-none",
      )}
      style={style}
    />
  );
}

/** The beat's sound along its foot, as bars. */
function BeatSound({
  label,
  peaks,
  px,
}: {
  label: string;
  peaks: number[];
  px: number;
}) {
  return (
    <span className="absolute inset-x-1 bottom-1 flex h-3.5 opacity-40">
      <Waveform
        aria-label={label}
        bars={Math.max(8, Math.round(px / 3))}
        className="size-full text-current"
        color="currentColor"
        peaks={peaks}
        progress={0}
      />
    </span>
  );
}

/** What happens at the beat, as a glyph; a moment's flag on its kind's
 * colour, in the ink that reads on it, so which kind it is reads at a
 * glance. */
function BeatGlyph({ beat }: { beat: TimelineBeat }) {
  const Glyph = BEAT_GLYPHS[beat.kind];
  if (beat.kind !== "moment")
    return <Glyph aria-hidden className="size-icon-sm shrink-0" />;
  return (
    <span
      className={cn(
        "flex size-4 shrink-0 items-center justify-center rounded-sm text-fg",
        isLight(beat.color) ? "appearance-light" : "appearance-dark",
      )}
      style={{ backgroundColor: beat.color }}
    >
      <Glyph aria-hidden className="size-icon-xs" />
    </span>
  );
}
