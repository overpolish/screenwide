// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Plus } from "lucide-react";
import { useRef, useState } from "react";

import { cn } from "../../../../lib/styling";
import { presetLabel, type SceneClip } from "../editor-data";
import { PresetDrawing } from "../island/preset-choice";
import { sceneRoom } from "../model/use-scenes";

import { at, placed, reach, type StripScale } from "./strip-scale";

// The scenes along the top of the strip, each over the stretch it frames:
// auto zoom's zooms to start with. Press one to choose it, drag its ends to
// retime it, and drag along the empty rail to draw a new zoom. Outside every
// scene, the recording keeps the composition it was recorded with. Over the
// empty rail, a faint zoom waits at the pointer, so the rail says what a
// drag there makes; dragged, it becomes the zoom it will be.

type Drag =
  | { from: number; kind: "draw"; to: number }
  | { edge: "end" | "start"; id: string; kind: "edge" };

/** The least room, in px, the waiting zoom is shown in: its icon's worth. */
const GHOST_MIN_PX = 20;

export function SceneRail({
  duration,
  onDraw,
  onRetime,
  onSelect,
  scale,
  scenes,
  secondsAt,
  selected,
  width,
}: {
  /** The recording's length, in seconds: the last scene's room ends there. */
  duration: number;
  onDraw: (from: number, to: number) => void;
  onRetime: (id: string, start: number, end: number) => void;
  onSelect: (id: string) => void;
  scale: StripScale;
  scenes: SceneClip[];
  /** The recorded moment under a pointer event. */
  secondsAt: (event: React.PointerEvent) => number;
  width: number;
  selected?: string;
}) {
  const dragRef = useRef<Drag | null>(null);
  const [draft, setDraft] = useState<{ from: number; to: number } | null>(null);
  const [hover, setHover] = useState<number | null>(null);
  // A new zoom fits between its neighbours, as the drawn one will, so what
  // is shown while drawing is what is made.
  const drawn = draft && {
    ...sceneRoom({ duration, scenes }, draft.from),
    end: Math.max(draft.from, draft.to),
    start: Math.min(draft.from, draft.to),
  };
  const hoverRoom =
    hover === null
      ? null
      : (scale.toX(sceneRoom({ duration, scenes }, hover).to) -
          scale.toX(hover)) *
        width;
  const move = (event: React.PointerEvent) => {
    const drag = dragRef.current;
    if (!drag) return;
    const moment = secondsAt(event);
    if (drag.kind === "draw") {
      dragRef.current = { ...drag, to: moment };
      setDraft({ from: drag.from, to: moment });
      return;
    }
    const clip = scenes.find(({ id }) => id === drag.id);
    if (!clip) return;
    if (drag.edge === "start") onRetime(clip.id, moment, clip.end);
    else onRetime(clip.id, clip.start, moment);
  };
  const end = () => {
    const drag = dragRef.current;
    dragRef.current = null;
    setDraft(null);
    if (drag?.kind === "draw") onDraw(drag.from, drag.to);
  };

  return (
    <div
      className="relative h-5 cursor-crosshair touch-none"
      dir="ltr"
      onPointerDown={(event) => {
        event.stopPropagation();
        event.currentTarget.setPointerCapture(event.pointerId);
        const moment = secondsAt(event);
        dragRef.current = { from: moment, kind: "draw", to: moment };
      }}
      onPointerLeave={() => {
        setHover(null);
      }}
      onPointerMove={(event) => {
        move(event);
        setHover(
          event.target === event.currentTarget ? secondsAt(event) : null,
        );
      }}
      onPointerUp={end}
      title="Drag to add a zoom"
    >
      {scenes.map((clip) => {
        const isSelected = clip.id === selected;
        const { pad, style } = reach(scale, clip, width);
        return (
          <button
            aria-label={`Scene, ${presetLabel(clip.preset)}`}
            aria-pressed={isSelected}
            className="group absolute inset-y-0 cursor-default"
            key={clip.id}
            onClick={() => {
              onSelect(clip.id);
            }}
            onPointerDown={(event) => {
              event.stopPropagation();
            }}
            style={style}
            type="button"
          >
            <span
              className={cn(
                "absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-[5px] px-1 text-caption whitespace-nowrap transition-colors",
                isSelected
                  ? "bg-primary text-on-primary"
                  : "bg-fill text-fg-secondary group-hover:bg-fill-strong",
              )}
              style={{ insetInlineEnd: pad, insetInlineStart: pad }}
            >
              <PresetDrawing id={clip.preset} />
              {presetLabel(clip.preset)}
            </span>
            {isSelected
              ? (["start", "end"] as const).map((edge) => (
                  <span
                    className="absolute inset-y-0 flex w-3 -translate-x-1/2 cursor-ew-resize items-center justify-center"
                    key={edge}
                    onPointerDown={(event) => {
                      event.stopPropagation();
                      event.currentTarget.setPointerCapture(event.pointerId);
                      dragRef.current = { edge, id: clip.id, kind: "edge" };
                    }}
                    onPointerMove={move}
                    onPointerUp={end}
                    style={{
                      insetInlineStart:
                        edge === "start"
                          ? pad
                          : `calc(100% - ${String(pad)}px)`,
                    }}
                  >
                    <span className="h-3.5 w-1 rounded-full bg-on-primary ring-2 ring-primary" />
                  </span>
                ))
              : null}
          </button>
        );
      })}
      {drawn ? (
        <span
          className="pointer-events-none absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-[5px] bg-fill-strong px-1 text-caption whitespace-nowrap text-fg"
          style={placed(
            scale,
            Math.max(drawn.from, drawn.start),
            Math.min(drawn.to, drawn.end),
          )}
        >
          <PresetDrawing id="zoom" />
          {presetLabel("zoom")}
        </span>
      ) : hover === null ||
        hoverRoom === null ||
        hoverRoom < GHOST_MIN_PX ? null : (
        <span
          className="pointer-events-none absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-[5px] border border-dashed border-fg-tertiary px-1 text-caption whitespace-nowrap text-fg-tertiary"
          style={{ ...at(scale, hover), maxWidth: hoverRoom }}
        >
          <Plus aria-hidden className="size-3 shrink-0" />
          {presetLabel("zoom")}
        </span>
      )}
    </div>
  );
}
