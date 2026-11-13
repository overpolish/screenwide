// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Music, Repeat } from "lucide-react";
import { useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { type BarPart } from "../bar-stack/use-stack-bar-drag";
import { RAIL_DRAG_PX, RailClip } from "../rail/rail-clip";
import { baseOf } from "../strip/strip-part";
import { type SnapFor } from "../strip/strip-snap";
import { type Span, type TimelineScale } from "../timeline-scale";

import { type TimelineAudioClip } from "./audio-bar";

// The songs along the foot of the strip, each a block over the stretch it
// plays, as the scenes are along its top (`rail-clip.tsx`). Songs never
// overlap, so one rail holds them all. Press one to choose it and drag it to
// move it whole, and drag a chosen one's ends to trim it; where it goes is
// the strip's to settle (`movedClip`): a song stops against the songs
// either side, or jumps past one pushed beyond its middle. With no key held
// its ends snap to the other songs' within reach, so songs are easily laid
// end to end; while ⌘ (Ctrl on Windows) is held they snap to what is in
// reach (`snap`) instead, and the strip draws its guide (`onGuide`). A
// press on the empty rail scrubs, as one anywhere on the strip does.

/** How near, in px, a song's edge comes to another song's before it snaps
 * to it, with no key held. */
const SONG_SNAP_PX = 8;

type Drag =
  | { edge: "end" | "start"; id: string; kind: "edge" }
  | {
      /** Where it was pressed, in recorded seconds and on screen. */
      grab: number;
      grabX: number;
      id: string;
      /** Whether the press has gone far enough to be a drag. */
      isDragging: boolean;
      kind: "move";
      /** The song as it was pressed. */
      moving: TimelineAudioClip;
    };

export type SongRailProps = {
  /** Told a song moved whole or an end of it pulled, in recorded seconds. */
  onRetime: (id: string, span: Span, part: BarPart) => void;
  onSelect: (id: string) => void;
  scale: TimelineScale;
  songs: TimelineAudioClip[];
  /** The strip's width, in px. */
  width: number;
  /** Where a dragged end has snapped while ⌘ or Ctrl is held, or null, for
   * the guide. */
  onGuide?: (seconds: number | null) => void;
  selected?: string;
  snap?: SnapFor;
};

export function SongRail({
  onGuide,
  onRetime,
  onSelect,
  scale,
  selected,
  snap,
  songs,
  width,
}: SongRailProps) {
  const ref = useRef<HTMLDivElement>(null);
  const dragRef = useRef<Drag | null>(null);
  /** Whether a pointer pressed a song, so the click that follows its
   * release does not choose it again. */
  const pressedRef = useRef(false);
  const secondsAt = (event: React.PointerEvent) => {
    const box = ref.current?.getBoundingClientRect();
    if (!box || box.width === 0) return 0;
    return scale.toSeconds((event.clientX - box.left) / box.width);
  };
  /** Where an end of song `id` at `seconds` settles: on what is in reach
   * while ⌘ or Ctrl is held, which `guide` tells, else on another song's
   * end within reach. */
  const settle = (id: string, seconds: number, isSnap: boolean) => {
    if (isSnap && snap) {
      const snapped = snap(baseOf(id))(seconds);
      return {
        guide: snapped.isSnapped ? snapped.seconds : null,
        seconds: snapped.seconds,
      };
    }
    const x = scale.toX(seconds) * width;
    let nearest: { distance: number; seconds: number } | null = null;
    for (const other of songs) {
      if (baseOf(other.id) === baseOf(id)) continue;
      for (const edge of [other.start, other.end]) {
        const distance = Math.abs(scale.toX(edge) * width - x);
        if (distance <= SONG_SNAP_PX && distance < (nearest?.distance ?? 1 / 0))
          nearest = { distance, seconds: edge };
      }
    }
    return { guide: null, seconds: nearest?.seconds ?? seconds };
  };
  const move = (event: React.PointerEvent) => {
    const drag = dragRef.current;
    if (!drag) return;
    const moment = secondsAt(event);
    const isSnap = event.metaKey || event.ctrlKey;
    if (drag.kind === "move") {
      if (
        !drag.isDragging &&
        Math.abs(event.clientX - drag.grabX) < RAIL_DRAG_PX
      )
        return;
      drag.isDragging = true;
      const { end, id, start } = drag.moving;
      const by = moment - drag.grab;
      // The start settles where it can, else the end.
      const head = settle(id, start + by, isSnap);
      const tail = settle(id, end + by, isSnap);
      const isHead = head.guide !== null || head.seconds !== start + by;
      const shift = isHead ? head.seconds - start : tail.seconds - end;
      onGuide?.(isHead ? head.guide : tail.guide);
      onRetime(id, { end: end + shift, start: start + shift }, "body");
      return;
    }
    const song = songs.find(({ id }) => id === drag.id);
    if (!song) return;
    const settled = settle(song.id, moment, isSnap);
    onGuide?.(settled.guide);
    onRetime(
      song.id,
      drag.edge === "start"
        ? { end: song.end, start: settled.seconds }
        : { end: settled.seconds, start: song.start },
      drag.edge,
    );
  };
  const end = () => {
    const drag = dragRef.current;
    dragRef.current = null;
    if (drag) onGuide?.(null);
    if (drag?.kind === "move") {
      // The click follows the release in the same task.
      setTimeout(() => {
        pressedRef.current = false;
      });
    }
  };

  return (
    <div
      className="relative h-5 touch-none rounded-part bg-fill/50"
      dir="ltr"
      onPointerMove={move}
      onPointerUp={end}
      ref={ref}
    >
      {songs.map((song) => (
        <RailClip
          glyph={
            <>
              <Music aria-hidden className="size-icon-sm shrink-0" />
              {song.isLooped ? (
                <Repeat aria-hidden className="size-icon-sm shrink-0" />
              ) : null}
            </>
          }
          isSelected={baseOf(song.id) === selected}
          key={song.id}
          label={t("editor-timeline-audio-song", { title: song.title })}
          name={song.title}
          onGrabBody={(event) => {
            pressedRef.current = true;
            dragRef.current = {
              grab: secondsAt(event),
              grabX: event.clientX,
              id: song.id,
              isDragging: false,
              kind: "move",
              moving: song,
            };
            onSelect(baseOf(song.id));
          }}
          onGrabEdge={(_, edge) => {
            dragRef.current = { edge, id: song.id, kind: "edge" };
          }}
          onPress={() => {
            // A pointer chose it as it was pressed; this is a keyboard's.
            if (!pressedRef.current) onSelect(baseOf(song.id));
          }}
          scale={scale}
          span={song}
          width={width}
        />
      ))}
    </div>
  );
}
