// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { cn } from "../../../../lib/styling";
import { type Editor } from "../model/use-editor";

import { type View } from "./strip-scale";
import { type ViewControls } from "./use-view";

// The whole recording under the strip, at the strip's scale: each beat a
// tick, the playhead, and a window round the part the strip shows. Drag
// the window to move along, its edges to zoom; press outside it to go
// there. Its length is the edited recording's against the recording's, so
// a squeeze visibly shortens it, and its end is where the squeeze is taken
// hold of.

type Drag = {
  edge: "end" | "move" | "start";
  view: View;
  width: number;
  x: number;
};

const share = (part: number) => `${String(part * 100)}%`;

export function Overview({
  controls,
  editor,
  onGrip,
}: {
  controls: ViewControls;
  editor: Editor;
  /** Whether the squeeze is held, while it changes the whole strip. */
  onGrip: (isGripping: boolean) => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const dragRef = useRef<Drag | null>(null);
  const gripRef = useRef<{ squeeze: number; width: number; x: number } | null>(
    null,
  );
  const [isDragging, setIsDragging] = useState(false);
  const { map } = editor;
  const { view } = controls;
  const at = (seconds: number) =>
    map.toTime(seconds) / Math.max(map.duration, 1e-6);

  const drag = (edge: Drag["edge"]) => ({
    onPointerDown: (event: React.PointerEvent) => {
      event.stopPropagation();
      event.currentTarget.setPointerCapture(event.pointerId);
      const width = ref.current?.getBoundingClientRect().width ?? 0;
      dragRef.current = { edge, view, width, x: event.clientX };
      setIsDragging(true);
    },
    onPointerMove: (event: React.PointerEvent) => {
      const held = dragRef.current;
      if (!held || held.width === 0) return;
      const by = (event.clientX - held.x) / held.width;
      controls.set({
        end: held.view.end + (held.edge === "start" ? 0 : by),
        start: held.view.start + (held.edge === "end" ? 0 : by),
      });
    },
    onPointerUp: () => {
      dragRef.current = null;
      setIsDragging(false);
    },
  });

  return (
    <div
      className="relative mt-1.5 h-3 touch-none"
      dir="ltr"
      onPointerDown={(event) => {
        // Pressed outside the window: the window goes there.
        const box = ref.current?.getBoundingClientRect();
        if (!box) return;
        const to = (event.clientX - box.left) / box.width;
        const span = view.end - view.start;
        controls.set({ end: to + span / 2, start: to - span / 2 });
      }}
      ref={ref}
      style={{
        width: share(map.duration / Math.max(map.recorded, map.duration)),
      }}
    >
      <span className="absolute inset-x-0 top-1/2 h-px -translate-y-1/2 bg-fill-strong" />
      {editor.beats.map((beat) => (
        <span
          aria-hidden
          className="absolute inset-y-0.5 min-w-px rounded-[1px] bg-fg-tertiary"
          key={beat.id}
          style={{
            insetInlineStart: share(at(beat.start)),
            width: share(at(beat.end) - at(beat.start)),
          }}
        />
      ))}
      <span
        aria-hidden
        className="absolute -inset-y-0.5 w-px bg-primary"
        style={{ insetInlineStart: share(at(editor.playhead)) }}
      />
      <span
        aria-hidden
        className={cn(
          "absolute -inset-y-1 cursor-grab rounded-[4px] border border-fg-secondary bg-fill transition-colors hover:border-fg",
          isDragging && "cursor-grabbing border-fg",
        )}
        style={{
          insetInlineStart: share(view.start),
          width: share(view.end - view.start),
        }}
        {...drag("move")}
      >
        <span
          className="absolute inset-y-0 -start-1 w-2 cursor-ew-resize"
          {...drag("start")}
        />
        <span
          className="absolute inset-y-0 -end-1 w-2 cursor-ew-resize"
          {...drag("end")}
        />
      </span>
      {/* The end is the recording's length: drag it in to squeeze. */}
      <button
        aria-label="Squeeze"
        className="absolute -end-4 -top-2 -bottom-2 flex w-4 cursor-ew-resize items-center justify-center"
        onDoubleClick={() => {
          editor.setSqueeze(0);
        }}
        onPointerDown={(event) => {
          event.stopPropagation();
          event.currentTarget.setPointerCapture(event.pointerId);
          const width =
            ref.current?.parentElement?.getBoundingClientRect().width ?? 0;
          gripRef.current = {
            squeeze: editor.squeeze,
            width,
            x: event.clientX,
          };
          onGrip(true);
        }}
        onPointerMove={(event) => {
          const grip = gripRef.current;
          if (!grip || grip.width === 0) return;
          const seconds =
            ((grip.x - event.clientX) / grip.width) * map.recorded;
          editor.setSqueeze(
            Math.min(Math.max(grip.squeeze + seconds, 0), map.maxSqueeze),
          );
        }}
        onPointerUp={() => {
          gripRef.current = null;
          onGrip(false);
        }}
        title="Drag in to squeeze the still time; double-click to undo"
        type="button"
      >
        <span className="h-5 w-1.5 rounded-full bg-fg-secondary" />
      </button>
    </div>
  );
}
