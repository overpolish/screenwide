// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { IconButton } from "../../../../primitives/button/icon-button";
import { IconSwap } from "../../../../primitives/icon-swap/icon-swap";
import { IslandKeyHint } from "../../../../primitives/island/island-key-hint";
import { useSize } from "../../../../primitives/island/island-size";
import { formatDuration } from "../../duration";

import {
  carried,
  contrasted,
  evenStart,
  fisheye,
  keptStart,
  keyedStart,
  windowPx,
} from "./fisheye";
import { FisheyeSound } from "./fisheye-sound";

// Where in its sound a clip plays from, chosen on the sound's waveform seen
// through a fisheye (`fisheye.ts`): the clip's stretch always magnified in
// its window, the rest of the sound squeezed either side. The window
// carries the lens: dragged, it follows the pointer, and the time it holds
// moves a pixel of its own magnified time per pixel near where it was
// taken, faster the further it is carried, so the track's width still
// crosses the whole song; with ⌘ (Ctrl on Windows) held it moves a
// quarter as far, for fine adjustment, which a hint under the waveform
// always says. Scrolled sideways, on a trackpad or with Shift and a wheel, it
// moves finely in place. A press beside the window starts it at the moment
// pressed, and the arrow keys move it a second at a time, a tenth with the
// same key held. The edit, and the preview with it, change once the window
// is let go or settles. Play hears the clip from there, as a moment's
// voice note plays, the timeline pausing under it; a line crosses the
// window as it plays. The waveform reads as time does, left to right in
// every language.

/** How far the window moves for each pixel of the pointer with ⌘ or Ctrl
 * held, against its own magnified time. */
const FINE_SHARE = 0.25;
/** How long after the last scroll the window settles where it is, in ms. */
const SETTLE_MS = 250;

export type SoundPreview = {
  isPlaying: boolean;
  /** How long the preview plays for, at most, in ms. */
  lengthMs: number;
  onTogglePlay: () => void;
  /** How far the preview has played, from 0 to 1. */
  progress: number;
};

export function SoundStartPicker({
  clipMs,
  latestMs,
  levels,
  offsetMs,
  onChange,
  preview,
  soundMs,
}: {
  /** How long the clip plays, in ms. */
  clipMs: number;
  /** The latest the clip can start in its sound, in ms. */
  latestMs: number;
  /** How loud the whole sound is, from 0 to 1, evenly along it. */
  levels: number[];
  offsetMs: number;
  onChange: (offsetMs: number) => void;
  preview: SoundPreview;
  soundMs: number;
}) {
  const [track, setTrack] = useState<HTMLDivElement | null>(null);
  const width = useSize(track)?.width ?? 0;
  /** Where the pointer took hold, with where the window started in the
   * sound and stood on the track then, where on the track the pointer was,
   * and whether ⌘ or Ctrl was held. */
  const grabRef = useRef<{
    fromMs: number;
    isFine: boolean;
    pointerX: number;
    startX: number;
    x: number;
  } | null>(null);
  const [dragged, setDragged] = useState<number | null>(null);
  const [isHeld, setIsHeld] = useState(false);
  // Let go, the window stays where it was put until the edit answers with
  // the clip's new start, so it never jumps back in between.
  const [answered, setAnswered] = useState(offsetMs);
  if (answered !== offsetMs) {
    setAnswered(offsetMs);
    if (!isHeld) setDragged(null);
  }
  const shownMs = dragged ?? offsetMs;
  const endMs = Math.min(shownMs + clipMs, soundMs);
  const windowWidth = windowPx(width, clipMs, soundMs);
  const room = width - windowWidth;
  // Where the window stands: where it was carried to, or else as far along
  // the track as it is along the sound.
  const [placedX, setPlacedX] = useState<number | null>(null);
  const startX = keptStart({
    atEnd: endMs >= soundMs,
    atStart: shownMs <= 0,
    room,
    startX:
      placedX ??
      evenStart({
        fromMs: shownMs,
        latestMs: soundMs - (endMs - shownMs),
        room,
      }),
  });
  const lens = fisheye({
    fromMs: shownMs,
    soundMs,
    startX,
    toMs: endMs,
    width,
    window: windowWidth,
  });
  const shown = useMemo(() => contrasted(levels), [levels]);
  const kept = (ms: number) => Math.round(Math.min(Math.max(ms, 0), latestMs));
  const windowStart = startX;
  const windowEnd = startX + windowWidth;
  const pointerX = (event: React.PointerEvent<HTMLElement>) =>
    event.clientX - event.currentTarget.getBoundingClientRect().left;
  // Scrolled sideways, the window moves finely and settles a moment after.
  const settleRef = useRef<number>(undefined);
  useEffect(
    () => () => {
      window.clearTimeout(settleRef.current);
    },
    [],
  );
  const heardMs = shownMs + preview.progress * preview.lengthMs;
  const label = t("editor-island-audio-plays-from");
  const sound = { label, lens, levels: shown, soundMs, width };
  return (
    <div className="flex flex-col gap-inline">
      <div className="flex items-center gap-inline">
        <IconButton
          aria-label={
            preview.isPlaying
              ? t("editor-island-audio-stop-preview")
              : t("editor-island-audio-preview")
          }
          className="shrink-0"
          onPress={preview.onTogglePlay}
        >
          <IconSwap swapKey={preview.isPlaying ? "pause" : "play"}>
            {/* Filled, as the editor's transport draws play and pause. */}
            {preview.isPlaying ? (
              <Pause aria-hidden className="size-icon fill-current" />
            ) : (
              <Play aria-hidden className="size-icon fill-current" />
            )}
          </IconSwap>
        </IconButton>
        <div
          className={cn(
            "relative h-island-control w-60 min-w-0 grow cursor-pointer touch-none",
            isHeld && "cursor-grabbing",
          )}
          dir="ltr"
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId);
            const x = pointerX(event);
            const inside = x >= windowStart && x <= windowEnd;
            // Pressed beside the window, it starts at the moment pressed.
            const fromMs = inside ? shownMs : kept(lens.toMs(x));
            const at = inside ? startX : x;
            setDragged(fromMs);
            setPlacedX(at);
            grabRef.current = {
              fromMs,
              isFine: event.metaKey || event.ctrlKey,
              pointerX: x,
              startX: Math.min(Math.max(at, 0), room),
              x: event.clientX,
            };
            setIsHeld(true);
          }}
          onPointerMove={(event) => {
            let grab = grabRef.current;
            if (grab === null) return;
            const isFine = event.metaKey || event.ctrlKey;
            // The key pressed or let go mid-drag takes hold afresh where the
            // window is, so it never jumps.
            if (isFine !== grab.isFine) {
              grab = {
                fromMs: shownMs,
                isFine,
                pointerX: pointerX(event),
                startX,
                x: event.clientX,
              };
              grabRef.current = grab;
            }
            const px = event.clientX - grab.x;
            const finePerPx = clipMs / Math.max(windowWidth, 1);
            // The sound's start and end are reached as the pointer reaches
            // the track's, wherever in the window it took hold.
            const moved = grab.isFine
              ? px * finePerPx * FINE_SHARE
              : carried({
                  finePerPx,
                  px,
                  remainingMs: px < 0 ? grab.fromMs : latestMs - grab.fromMs,
                  roomPx: px < 0 ? grab.pointerX : width - grab.pointerX,
                });
            setDragged(kept(grab.fromMs + moved));
            setPlacedX(grab.startX + (grab.isFine ? px * FINE_SHARE : px));
          }}
          onPointerUp={() => {
            grabRef.current = null;
            setIsHeld(false);
            if (dragged !== null && dragged !== offsetMs) onChange(dragged);
            else setDragged(null);
          }}
          onWheel={(event) => {
            const deltaPx = event.deltaX || (event.shiftKey ? event.deltaY : 0);
            if (deltaPx === 0 || grabRef.current !== null) return;
            // A pixel of the window's own magnified time at a time.
            const next = kept(
              shownMs + deltaPx * (clipMs / Math.max(windowWidth, 1)),
            );
            setDragged(next);
            window.clearTimeout(settleRef.current);
            settleRef.current = window.setTimeout(() => {
              if (next !== offsetMs) onChange(next);
            }, SETTLE_MS);
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
              clipPath: `inset(0 ${String(width - windowEnd)}px 0 ${String(windowStart)}px)`,
            }}
          >
            <FisheyeSound {...sound} />
          </div>
          <div
            aria-label={label}
            aria-valuemax={latestMs}
            aria-valuemin={0}
            aria-valuenow={shownMs}
            aria-valuetext={formatDuration(shownMs)}
            className="absolute inset-y-0 cursor-grab rounded-sm ring-1 ring-fg-secondary outline-none ring-inset focus-visible:ring-2 focus-visible:ring-focus"
            onKeyDown={(event) => {
              const to = keyedStart({
                isFine: event.metaKey || event.ctrlKey,
                key: event.key,
                latestMs,
                offsetMs,
              });
              if (to === null) return;
              event.preventDefault();
              onChange(to);
            }}
            role="slider"
            style={{
              insetInlineStart: windowStart,
              width: windowEnd - windowStart,
            }}
            tabIndex={0}
          />
          {preview.isPlaying ? (
            <span
              aria-hidden
              className="pointer-events-none absolute inset-y-0 w-0.5 rounded-full bg-primary"
              style={{ insetInlineStart: lens.toX(heardMs) }}
            />
          ) : null}
        </div>
      </div>
      <IslandKeyHint
        binding="CommandOrControl"
        message={(key) => t("editor-island-audio-fine-hint", { key })}
      />
    </div>
  );
}
