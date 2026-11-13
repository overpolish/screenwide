// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { useSize } from "../../../../primitives/island/island-size";
import { formatDuration } from "../../duration";
import { LEAST_CLIP_MS } from "../../sound-track/audio-clips";
import { IslandPlayButton } from "../island-play-button";

import { contrasted, keyedStart, type Lens } from "./fisheye";
import { FisheyeSound } from "./fisheye-sound";
import { type SoundPreview } from "./sound-start-picker";

// The stretch of a sound effect its clip plays, chosen on the whole sound's
// waveform: a sound effect is short enough to show whole, evenly, so there
// is no fisheye. A window frames the stretch: dragged, it moves through the
// sound as long as it is; either end dragged trims that end; pressed beside
// it, it starts at the moment pressed. The arrow keys move it a second at a
// time, a tenth with ⌘ (Ctrl on Windows) held. The edit, and the preview
// with it, change once the window is let go. Play hears the stretch, a
// line crossing the window as it plays. The waveform reads as time does,
// left to right in every language.

/** The stretch the window frames, in ms of the sound. */
type Stretch = { fromMs: number; toMs: number };

type Part = "body" | "end" | "start";

export function SoundTrimPicker({
  clipMs,
  levels,
  offsetMs,
  onChange,
  preview,
  soundMs,
}: {
  /** How long the clip plays, in ms. */
  clipMs: number;
  /** How loud the whole sound is, from 0 to 1, evenly along it. */
  levels: number[];
  offsetMs: number;
  onChange: (stretch: { lengthMs: number; offsetMs: number }) => void;
  preview: SoundPreview;
  soundMs: number;
}) {
  const [track, setTrack] = useState<HTMLDivElement | null>(null);
  const width = useSize(track)?.width ?? 0;
  const lens: Lens = {
    toMs: (x) => (width > 0 ? (x / width) * soundMs : 0),
    toX: (ms) => (soundMs > 0 ? (ms / soundMs) * width : 0),
  };
  const playing = {
    fromMs: offsetMs,
    toMs: Math.min(offsetMs + clipMs, soundMs),
  };
  /** Where the pointer took hold, the stretch then, and what of it. */
  const grabRef = useRef<{ part: Part; stretch: Stretch; x: number } | null>(
    null,
  );
  const [dragged, setDragged] = useState<Stretch | null>(null);
  // Let go, the window stays where it was put until the edit answers with
  // the clip's new stretch, so it never jumps back in between.
  const [answered, setAnswered] = useState(playing);
  if (answered.fromMs !== playing.fromMs || answered.toMs !== playing.toMs) {
    setAnswered(playing);
    if (grabRef.current === null) setDragged(null);
  }
  const shown = dragged ?? playing;
  const levelsShown = useMemo(() => contrasted(levels), [levels]);
  const pointerX = (event: React.PointerEvent<HTMLElement>) =>
    event.clientX - event.currentTarget.getBoundingClientRect().left;
  /** The stretch `grab` holds moved by `ms`, as what was taken says. */
  const moved = (
    part: Part,
    { fromMs, toMs }: Stretch,
    ms: number,
  ): Stretch => {
    if (part === "start")
      return {
        fromMs: Math.min(Math.max(fromMs + ms, 0), toMs - LEAST_CLIP_MS),
        toMs,
      };
    if (part === "end")
      return {
        fromMs,
        toMs: Math.max(Math.min(toMs + ms, soundMs), fromMs + LEAST_CLIP_MS),
      };
    const length = toMs - fromMs;
    const from = Math.min(Math.max(fromMs + ms, 0), soundMs - length);
    return { fromMs: from, toMs: from + length };
  };
  const startX = lens.toX(shown.fromMs);
  const endX = lens.toX(shown.toMs);
  const sound = {
    label: t("editor-island-audio-plays"),
    lens,
    levels: levelsShown,
    soundMs,
    width,
  };
  const heardMs = shown.fromMs + preview.progress * preview.lengthMs;
  return (
    <div className="flex items-center gap-inline">
      <IslandPlayButton
        isPlaying={preview.isPlaying}
        label={
          preview.isPlaying
            ? t("editor-island-audio-stop-preview")
            : t("editor-island-audio-preview")
        }
        onPress={preview.onTogglePlay}
      />
      <div
        className={cn(
          "relative h-island-control w-60 min-w-0 grow cursor-pointer touch-none",
          dragged !== null && "cursor-grabbing",
        )}
        dir="ltr"
        onPointerDown={(event) => {
          event.currentTarget.setPointerCapture(event.pointerId);
          const x = pointerX(event);
          const target = event.target as HTMLElement;
          const part = (target.dataset.part as Part | undefined) ?? "body";
          const inside = x >= startX && x <= endX;
          // Pressed beside the window, it starts at the moment pressed.
          const stretch =
            part !== "body" || inside
              ? shown
              : moved("body", shown, lens.toMs(x) - shown.fromMs);
          grabRef.current = { part, stretch, x };
          setDragged(stretch);
        }}
        onPointerMove={(event) => {
          const grab = grabRef.current;
          if (grab === null) return;
          setDragged(
            moved(grab.part, grab.stretch, lens.toMs(pointerX(event) - grab.x)),
          );
        }}
        onPointerUp={() => {
          grabRef.current = null;
          if (
            dragged !== null &&
            (dragged.fromMs !== playing.fromMs || dragged.toMs !== playing.toMs)
          )
            onChange({
              lengthMs: Math.round(dragged.toMs - dragged.fromMs),
              offsetMs: Math.round(dragged.fromMs),
            });
          else setDragged(null);
        }}
        ref={setTrack}
      >
        <div aria-hidden className="pointer-events-none absolute inset-0">
          <FisheyeSound {...sound} />
        </div>
        {/* The stretch the clip plays, drawn bright over the faint whole. */}
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 text-fg"
          style={{
            clipPath: `inset(0 ${String(width - endX)}px 0 ${String(startX)}px)`,
          }}
        >
          <FisheyeSound {...sound} />
        </div>
        <div
          aria-label={t("editor-island-audio-plays")}
          aria-valuemax={soundMs - (shown.toMs - shown.fromMs)}
          aria-valuemin={0}
          aria-valuenow={shown.fromMs}
          aria-valuetext={t("editor-island-audio-plays-value", {
            from: formatDuration(shown.fromMs),
            length: formatDuration(shown.toMs - shown.fromMs),
          })}
          className="absolute inset-y-0 cursor-grab rounded-sm ring-1 ring-fg-secondary outline-none ring-inset focus-visible:ring-2 focus-visible:ring-focus"
          onKeyDown={(event) => {
            const length = playing.toMs - playing.fromMs;
            const to = keyedStart({
              isFine: event.metaKey || event.ctrlKey,
              key: event.key,
              latestMs: soundMs - length,
              offsetMs,
            });
            if (to === null) return;
            event.preventDefault();
            onChange({ lengthMs: length, offsetMs: to });
          }}
          role="slider"
          style={{ insetInlineStart: startX, width: endX - startX }}
          tabIndex={0}
        >
          {(["start", "end"] as const).map((part) => (
            <span
              aria-hidden
              className={cn(
                "absolute inset-y-0 w-3 cursor-ew-resize",
                part === "start" ? "-start-1.5" : "-end-1.5",
              )}
              data-part={part}
              key={part}
            >
              <span className="pointer-events-none absolute inset-y-1 start-1/2 w-1.5 -translate-x-1/2 rounded-full bg-fg ring-2 ring-recess" />
            </span>
          ))}
        </div>
        {preview.isPlaying ? (
          <span
            aria-hidden
            className="pointer-events-none absolute inset-y-0 w-0.5 rounded-full bg-primary"
            style={{ insetInlineStart: lens.toX(heardMs) }}
          />
        ) : null}
      </div>
    </div>
  );
}
