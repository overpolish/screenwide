// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { cn } from "../../../../lib/styling";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { captionLines, focusOf, isOverlayHidden } from "../model/strip-items";
import { type Editor } from "../model/use-editor";
import { beatAt } from "../recording";

import { MarkRail, type Snap } from "./mark-rail";
import { Overview } from "./overview";
import { Pauses } from "./pauses";
import { Blade, RangeBand } from "./range-cut";
import { SceneRail } from "./scene-rail";
import { BeatBlock, Seams, VoiceRow } from "./strip-beats";
import { MomentPreview, Playhead } from "./strip-rails";
import { placed, stripScale, type View } from "./strip-scale";
import { useRangeCut } from "./use-range-cut";
import { useScrollView } from "./use-scroll-view";
import { useView } from "./use-view";

// The timeline as one strip of what happened, to scale, showing the part of
// the recording in view; the overview under it shows the whole. Along its
// top are the scenes, over the beats the annotations, and under them what
// was done, each beat as the screen at that moment, with the pauses between
// as springs.
//
// Pressing a beat, a pause, a scene or an annotation selects it, and nothing
// else moves; pressing the empty strip moves the playhead there; dragging
// the playhead's knob scrubs. Hovering only shows the moment under the
// pointer.

/** How close, in px, an annotation's end must come to an event to snap. */
const SNAP_PX = 8;
/** The whole recording: the track inside the scroll area draws it all. */
const WHOLE: View = { end: 1, start: 0 };

function useWidth() {
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const element = ref.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      setWidth(entry.contentRect.width);
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);
  return { ref, width };
}

export function ActivityStrip({ editor }: { editor: Editor }) {
  const frame = useWidth();
  const trackRef = useRef<HTMLDivElement>(null);
  const { map, selection } = editor;
  const controls = useView({ duration: map.duration, width: frame.width });
  const { onInitialized, trackWidth } = useScrollView(controls, frame.width);
  // Along the track, which is the whole recording; and across the part in
  // view, for what is drawn over the scroll area rather than in it.
  const scale = stripScale({ map, view: WHOLE });
  const viewScale = stripScale({ map, view: controls.view });
  const [isGripping, setIsGripping] = useState(false);
  const [hoverX, setHoverX] = useState<number | null>(null);
  // Pressed on the empty strip, and held: the playhead follows the pointer.
  const isScrubbingRef = useRef(false);
  const share = (seconds: number) =>
    map.toTime(seconds) / Math.max(map.duration, 1e-6);

  // What is chosen is brought into view if it is out of it, and the
  // playhead is followed while it plays. Only a new choice moves the view,
  // never an edit to what is chosen, so the effects read the rest through
  // a ref.
  const focus = focusOf(editor, selection);
  const latestRef = useRef({ controls, focus, share });
  useEffect(() => {
    latestRef.current = { controls, focus, share };
  });
  useEffect(() => {
    const latest = latestRef.current;
    if (latest.focus === null) return;
    const middle =
      (latest.share(latest.focus.start) + latest.share(latest.focus.end)) / 2;
    latest.controls.reveal(middle, middle);
  }, [selection]);
  useEffect(() => {
    if (!editor.isPlaying) return;
    const latest = latestRef.current;
    const now = latest.share(editor.playhead);
    latest.controls.reveal(now, now);
  }, [editor.playhead, editor.isPlaying]);

  const fractionOf = (event: React.PointerEvent | React.MouseEvent) => {
    const box = trackRef.current?.getBoundingClientRect();
    return box && box.width > 0 ? (event.clientX - box.left) / box.width : 0;
  };
  const secondsAt = (event: React.PointerEvent) =>
    scale.toSeconds(fractionOf(event));
  const scrub = (event: React.PointerEvent) => {
    editor.seek(secondsAt(event));
  };
  const range = useRangeCut({ editor, fractionOf, secondsAt });
  const events = [
    ...editor.beats.flatMap(({ end, start }) => [start, end]),
    ...editor.recording.actions.map(({ at: moment }) => moment),
    editor.playhead,
  ];
  const snap: Snap = (seconds) => {
    const near = events.reduce((best, event) =>
      Math.abs(event - seconds) < Math.abs(best - seconds) ? event : best,
    );
    const isSnapped =
      Math.abs(scale.toX(near) - scale.toX(seconds)) * trackWidth < SNAP_PX;
    return { isSnapped, seconds: isSnapped ? near : seconds };
  };
  const selectedBeat = selection?.kind === "beat" ? selection.id : undefined;
  const hasScenes = editor.kind === "recording";

  return (
    <div className="relative min-w-0 grow pe-5">
      <div className="relative" dir="ltr" ref={frame.ref}>
        <ScrollArea
          className="px-2"
          dir="ltr"
          onInitialized={onInitialized}
          orientation="horizontal"
          scrollbarHidden
        >
          <div
            // The playhead's knob has a band of its own along the top,
            // above the scenes, so it is never taken for part of one.
            className={cn(
              "relative touch-none pb-1",
              hasScenes ? "pt-8" : "pt-3",
            )}
            onClickCapture={range.handlers.onClickCapture}
            onPointerDown={(event) => {
              event.currentTarget.setPointerCapture(event.pointerId);
              isScrubbingRef.current = true;
              editor.seek(secondsAt(event));
              // A selection and a mode never stand together, so letting the
              // selection go here leaves an open mode alone.
              if (selection) editor.select(null);
            }}
            onPointerDownCapture={range.handlers.onPointerDownCapture}
            onPointerLeave={() => {
              setHoverX(null);
              range.handlers.onPointerLeave();
            }}
            onPointerMove={(event) => {
              if (isScrubbingRef.current) {
                editor.seek(secondsAt(event));
                return;
              }
              // Dragging, the frame would cover what is being dragged; over a
              // beat or a pause, the strip already shows it.
              const isOnItem = (event.target as Element).closest(
                "button, [role='button']",
              );
              setHoverX(event.buttons || isOnItem ? null : fractionOf(event));
            }}
            onPointerMoveCapture={range.handlers.onPointerMoveCapture}
            onPointerUp={() => {
              isScrubbingRef.current = false;
            }}
            onPointerUpCapture={range.handlers.onPointerUpCapture}
            ref={trackRef}
            style={{ width: trackWidth }}
          >
            {focus === null ? null : (
              // Spans what is chosen as drawn, so the island stands over its
              // middle, wherever a squeeze or a trim leaves that.
              <span
                aria-hidden
                className="pointer-events-none absolute top-0"
                data-island-anchor="selection"
                style={placed(scale, focus.start, focus.end)}
              />
            )}
            {hasScenes ? (
              <div className="absolute inset-x-0 top-3 z-10">
                <SceneRail
                  duration={editor.recording.duration}
                  onDraw={editor.drawScene}
                  onRetime={editor.retimeScene}
                  onSelect={(id) => {
                    editor.select({ id, kind: "scene" });
                  }}
                  scale={scale}
                  scenes={editor.scenes}
                  secondsAt={secondsAt}
                  selected={
                    selection?.kind === "scene" ? selection.id : undefined
                  }
                  width={trackWidth}
                />
              </div>
            ) : null}
            <MarkRail
              marks={editor.kind === "recording" ? editor.marks : []}
              onRetime={editor.retimeMark}
              onSelect={(id) => {
                editor.select({ id, kind: "mark" });
              }}
              scale={scale}
              secondsAt={secondsAt}
              selected={selection?.kind === "mark" ? selection.id : undefined}
              snap={snap}
              width={trackWidth}
            />
            <div className="relative h-11">
              <Pauses editor={editor} scale={scale} width={trackWidth} />
              {editor.beats.map((beat) => (
                <BeatBlock
                  beat={beat}
                  isHidden={isOverlayHidden(editor, beat)}
                  isSelected={beat.id === selectedBeat}
                  key={beat.id}
                  onPress={() => {
                    editor.select({ id: beat.id, kind: "beat" });
                    editor.seek(beatAt(beat));
                  }}
                  recording={editor.recording}
                  scale={scale}
                  width={trackWidth}
                />
              ))}
              <Seams
                cuts={editor.cuts}
                onRestore={editor.restore}
                scale={scale}
              />
              {range.draft ? (
                <RangeBand scale={scale} stretch={range.draft} />
              ) : selection?.kind === "range" ? (
                <RangeBand scale={scale} stretch={selection} />
              ) : null}
            </div>
            {editor.isVoiced && editor.kind === "recording" ? (
              <div className="mt-1">
                <VoiceRow
                  isHidden={!editor.effects.captions}
                  isOpen={editor.mode === "captions"}
                  lines={captionLines(editor)}
                  // The playhead goes to the line, so the caption being
                  // styled is on the picture.
                  onPress={(line) => {
                    editor.seek(line.start);
                    editor.setMode("captions", line);
                  }}
                  recording={editor.recording}
                  scale={scale}
                  width={trackWidth}
                />
              </div>
            ) : null}
            {/* Over the scenes, which its line crosses, but pressed only by
              its knob. */}
            <span className="pointer-events-none absolute inset-x-0 top-1.5 bottom-0 z-20">
              <Playhead
                onScrub={scrub}
                scale={scale}
                seconds={editor.playhead}
              />
            </span>
            {range.bladeX === null ? null : <Blade x={range.bladeX} />}
          </div>
        </ScrollArea>
        {/* Over the scroll area, which clips what rises above it. With
            something chosen, the island stands over the strip, and the
            frame would cover it. */}
        {hoverX !== null &&
        !isGripping &&
        !selection &&
        range.bladeX === null ? (
          <div className="pointer-events-none absolute inset-x-2 top-0">
            <MomentPreview
              recording={editor.recording}
              scale={viewScale}
              seconds={scale.toSeconds(hoverX)}
            />
          </div>
        ) : null}
      </div>
      <Overview controls={controls} editor={editor} onGrip={setIsGripping} />
    </div>
  );
}
