// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { cn } from "../../../../lib/styling";
import { cursorAt } from "../../ideas/cursor-path";
import { type Element } from "../../ideas/ideas-data";
import { corner, newMark, type Point } from "../../ideas/look-pens-model";
import { MockApp } from "../../ideas/mock-app";
import { PlacedMarks } from "../../ideas/placed-marks";
import { elementAt, H, nearestEdge, SNAP, W } from "../../ideas/point-geometry";
import { RecordedCursor } from "../../ideas/recorded-cursor";
import { ScreenRipple } from "../../ideas/screen-ripple";
import { backdropOf, stepAt } from "../editor-data";
import { type Editor } from "../model/use-editor";

import { Camera, Caption, Handles, LockOn } from "./picture-layers";

// The work: the recorded screen at the playhead, with the annotations that
// show then, drawn in the project's look. Click one to select it; with a
// pen in hand, what the pointer would land on is outlined and a click (or a
// drag, for an arrow) places it, Point, don't draw's way. A step shown as
// Follow pointer leans in toward the recording's pointer.

type Drag = { from: Point; to: Point; snapped?: Element };

export function WorkPicture({ editor }: { editor: Editor }) {
  const appRef = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<Element | undefined>();
  const [drag, setDrag] = useState<Drag | null>(null);
  const [ripple, setRipple] = useState<{ centre: Point; id: number } | null>(
    null,
  );
  const { kind, playhead, tool } = editor;
  const isRecording = kind === "recording";
  const isPen = tool !== "select";
  const marks =
    kind === "screenshot"
      ? editor.marks
      : editor.marks.filter(
          ({ end, start }) => playhead >= start && playhead < end,
        );
  const { selection } = editor;
  const selected =
    selection?.kind === "mark"
      ? marks.find(({ id }) => id === selection.id)
      : undefined;
  const layout = isRecording ? editor.scenes[stepAt(playhead)] : "screen";
  const pointerAt = cursorAt(playhead);
  const hasBackdrop = editor.backdrop !== "none";

  // The app layer may be scaled, by a backdrop or a scene leaning in, so a
  // point is read against its drawn box and brought back to its own size.
  const pointer = (event: React.PointerEvent): Point => {
    const box = appRef.current?.getBoundingClientRect();
    if (!box) return { x: 0, y: 0 };
    return {
      x: ((event.clientX - box.left) / box.width) * W,
      y: ((event.clientY - box.top) / box.height) * H,
    };
  };

  const place = (mark: ReturnType<typeof newMark>) => {
    editor.place(mark);
    setHover(undefined);
    if (mark.pen === "counter")
      setRipple({ centre: corner(mark.element).at, id: mark.id });
  };

  return (
    <div
      className="relative shrink-0 overflow-hidden rounded-part shadow-lg"
      dir="ltr"
      style={{ background: backdropOf(editor.backdrop), height: H, width: W }}
    >
      <div
        className={cn(
          "absolute inset-0 touch-none overflow-hidden transition-transform duration-700 ease-out",
          isPen ? "cursor-crosshair" : "cursor-default",
          hasBackdrop && "rounded-[10px] shadow-2xl",
        )}
        onPointerDown={(event) => {
          const at = pointer(event);
          if (!isPen) {
            editor.select(null);
            return;
          }
          if (tool === "arrow") {
            event.currentTarget.setPointerCapture(event.pointerId);
            setDrag({ from: at, to: at });
            return;
          }
          const element = elementAt(at.x, at.y);
          if (element) place(newMark({ element, id: Date.now(), pen: tool }));
        }}
        onPointerLeave={() => {
          setHover(undefined);
        }}
        onPointerMove={(event) => {
          const at = pointer(event);
          if (drag) {
            const near = nearestEdge(at.x, at.y);
            const snapped = near.distance < SNAP ? near : undefined;
            setDrag({
              ...drag,
              snapped: snapped?.element,
              to: snapped ? snapped.point : at,
            });
            return;
          }
          setHover(isPen ? elementAt(at.x, at.y) : undefined);
        }}
        onPointerUp={() => {
          if (drag?.snapped) {
            place(
              newMark({
                element: drag.snapped,
                from: drag.from,
                id: Date.now(),
                pen: "arrow",
                to: drag.to,
              }),
            );
          }
          setDrag(null);
        }}
        ref={appRef}
        style={{
          transform: hasBackdrop
            ? "scale(0.88)"
            : layout === "follow" && editor.isPlaying
              ? "scale(1.18)"
              : "none",
          transformOrigin:
            layout === "follow" && !hasBackdrop
              ? `${String(pointerAt.x)}% ${String(pointerAt.y)}%`
              : "50% 50%",
        }}
      >
        <MockApp className="size-full" />
        {ripple ? (
          <ScreenRipple centre={ripple.centre} key={ripple.id}>
            <MockApp className="size-full" />
          </ScreenRipple>
        ) : null}
        <PlacedMarks
          draft={drag}
          isSelecting={!isPen}
          lookId={editor.lookId}
          marks={marks}
          onSelect={(id) => {
            editor.select({ id, kind: "mark" });
          }}
        />
        {selected ? <Handles mark={selected} /> : null}
        {isPen && (drag?.snapped ?? hover) ? (
          <LockOn element={(drag?.snapped ?? hover) as Element} />
        ) : null}
        {isRecording ? (
          <RecordedCursor
            hasRipple={editor.effects.ripple}
            seconds={playhead}
          />
        ) : null}
      </div>
      {isRecording ? <Camera layout={layout} /> : null}
      {isRecording && editor.isVoiced && editor.effects.captions ? (
        <Caption seconds={playhead} />
      ) : null}
    </div>
  );
}
