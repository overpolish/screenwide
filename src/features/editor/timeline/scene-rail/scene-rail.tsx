// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Plus } from "lucide-react";
import { useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { at, placed, type Span, type TimelineScale } from "../timeline-scale";

import { SceneClip, type TimelineScene } from "./scene-clip";

// The scenes along the top of the strip, each over the stretch it frames,
// drawn as its layout and named. Press one to choose it, drag a chosen
// one's ends to retime it, and drag along the empty rail to draw a new one.
// Outside every scene, the recording keeps the composition it was recorded
// with. Over the empty rail, a faint scene waits at the pointer, so the
// rail says what a drag there makes; dragged, it becomes the scene it will
// be, fitted between its neighbours as the drawn one will be.

/** The least room, in px, the waiting scene is shown in: its glyph's. */
const GHOST_MIN_PX = 20;

type Drag =
  | { edge: "end" | "start"; id: string; kind: "edge" }
  | { from: number; kind: "draw"; to: number };

/** The room a scene at a moment can have: from the end of the one before
 * to the start of the one after, within the recording. */
function sceneRoom(scenes: Span[], duration: number, moment: number) {
  return {
    from: Math.max(
      0,
      ...scenes.filter(({ end }) => end <= moment).map(({ end }) => end),
    ),
    to: Math.min(
      duration,
      ...scenes
        .filter(({ start }) => start >= moment)
        .map(({ start }) => start),
    ),
  };
}

export type SceneRailProps = {
  /** The recording's length, in recorded seconds: the last scene's room
   * ends there. */
  duration: number;
  /** Told a stretch drawn along the empty rail, from where it was pressed
   * to where it was let go, either way round. */
  onDraw: (from: number, to: number) => void;
  onRetime: (id: string, start: number, end: number) => void;
  onSelect: (id: string) => void;
  scale: TimelineScale;
  scenes: TimelineScene[];
  /** The strip's width, in px. */
  width: number;
  selected?: string;
};

export function SceneRail({
  duration,
  onDraw,
  onRetime,
  onSelect,
  scale,
  scenes,
  selected,
  width,
}: SceneRailProps) {
  const ref = useRef<HTMLDivElement>(null);
  const dragRef = useRef<Drag | null>(null);
  const [draft, setDraft] = useState<{ from: number; to: number } | null>(null);
  const [hover, setHover] = useState<number | null>(null);
  const secondsAt = (event: React.PointerEvent) => {
    const box = ref.current?.getBoundingClientRect();
    if (!box || box.width === 0) return 0;
    return scale.toSeconds((event.clientX - box.left) / box.width);
  };
  const drawn = draft && {
    ...sceneRoom(scenes, duration, draft.from),
    end: Math.max(draft.from, draft.to),
    start: Math.min(draft.from, draft.to),
  };
  const hoverRoom =
    hover === null
      ? null
      : (scale.toX(sceneRoom(scenes, duration, hover).to) - scale.toX(hover)) *
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
    const scene = scenes.find(({ id }) => id === drag.id);
    if (!scene) return;
    if (drag.edge === "start") onRetime(scene.id, moment, scene.end);
    else onRetime(scene.id, scene.start, moment);
  };
  const end = () => {
    const drag = dragRef.current;
    dragRef.current = null;
    setDraft(null);
    // What was drawn now sits under the pointer; the waiting scene comes
    // back with the next move over the empty rail.
    setHover(null);
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
        // While held, the rail has the pointer, wherever it is.
        setHover(
          !dragRef.current && event.target === event.currentTarget
            ? secondsAt(event)
            : null,
        );
      }}
      onPointerUp={end}
      ref={ref}
    >
      {scenes.map((scene) => (
        <SceneClip
          isSelected={scene.id === selected}
          key={scene.id}
          onGrabEdge={(_, edge) => {
            dragRef.current = { edge, id: scene.id, kind: "edge" };
          }}
          onPress={() => {
            onSelect(scene.id);
          }}
          scale={scale}
          scene={scene}
          width={width}
        />
      ))}
      {drawn ? (
        <span
          aria-hidden
          className="pointer-events-none absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-part bg-fill-strong px-1 text-caption whitespace-nowrap text-fg"
          style={placed(
            scale,
            Math.max(drawn.from, drawn.start),
            Math.min(drawn.to, drawn.end),
          )}
        >
          <Plus aria-hidden className="size-icon-sm shrink-0" />
          {t("editor-timeline-scene-new")}
        </span>
      ) : hover === null ||
        hoverRoom === null ||
        hoverRoom < GHOST_MIN_PX ? null : (
        <span
          aria-hidden
          className="pointer-events-none absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-part border border-dashed border-fg-tertiary px-1 text-caption whitespace-nowrap text-fg-tertiary"
          style={{ ...at(scale, hover), maxWidth: hoverRoom }}
        >
          <Plus aria-hidden className="size-icon-sm shrink-0" />
          {t("editor-timeline-scene-new")}
        </span>
      )}
    </div>
  );
}
