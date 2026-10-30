// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motionDurationCss } from "../../../../lib/motion";
import { cn } from "../../../../lib/styling";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import {
  AnnotationLane,
  type TimelineAnnotation,
} from "../annotation-lane/annotation-lane";
import { CaptionLane } from "../caption-lane/caption-lane";
import { type TimelineCaption } from "../caption-lane/caption-line";
import { Minimap } from "../minimap/minimap";
import { TimelinePlayhead } from "../playhead/timeline-playhead";
import { CutBlade } from "../range-cut/cut-blade";
import { type TimelineScene } from "../scene-rail/scene-clip";
import { SceneRail } from "../scene-rail/scene-rail";
import { type Playhead } from "../scrub-playhead";
import { type Span, type TimeMap, timelineScale } from "../timeline-scale";

import { beatEdgeSnap } from "./beat-edge-snap";
import { JumpMarks } from "./jump-marks";
import {
  type StripBeat,
  StripBeatRow,
  type StripBeatRowProps,
  type StripLull,
} from "./strip-beat-row";
import { selectionSpan, type StripSelection } from "./strip-selection";
import { splitParts } from "./strip-split";
import { useRangeCut } from "./use-range-cut";
import { useStripReorder } from "./use-strip-reorder";
import { useStripScrub } from "./use-strip-scrub";
import { useStripView } from "./use-strip-view";
import { useWidth } from "./use-width";

// The timeline as one strip of what happened, to scale, showing the part of
// the recording in view; the minimap under it shows the whole. Along its
// top are the scenes, under them the annotations, then what was done, each
// beat as the screen at that moment with the lulls between as springs, and
// under the beats the voice's captions. The playhead crosses them all.
//
// Pressing a part selects it, and nothing else moves; pressing the empty
// strip moves the playhead there; dragging the playhead's knob scrubs.
// With ⌥ held, a drag across the strip draws a stretch to cut. A beat, a
// lull, what is chosen or the scissors' stretch, dragged along the strip,
// moves to play elsewhere, the rest making room for it as it goes.

/** How the parts and the minimap's ticks glide to new places as the strip
 * makes room; what was lifted, let go, glides down into its own. */
const GLIDE =
  "[&_*]:transition-[left,right,width,transform,translate] [&_*]:duration-(--glide) [&_*]:ease-out";

export type TimelineStripProps = Pick<
  StripBeatRowProps,
  | "cuts"
  | "isAltHeld"
  | "onBeatTrim"
  | "onLullPlayChange"
  | "onLullTrim"
  | "onRestore"
  | "onSeek"
  | "pictureAspect"
  | "selection"
> & {
  annotations: TimelineAnnotation[];
  beats: StripBeat[];
  /** The voice's lines, or null where the recording has no voice. */
  captions: TimelineCaption[] | null;
  /** Whether the captions are hidden from the picture. */
  isCaptionsHidden: boolean;
  /** Whether the recording plays, which the view follows. */
  isPlaying: boolean;
  /** The recorded moments where play jumps, each the start of a stretch
   * that does not follow what plays before it. */
  jumps: number[];
  lulls: StripLull[];
  /** How the recording plays now. */
  map: TimeMap;
  /** The most the lulls can be squeezed by, in seconds. */
  maxSqueeze: number;
  onAnnotationRestack: (lanes: Record<string, number>) => void;
  onAnnotationRetime: (id: string, start: number, end: number) => void;
  onCaptionPress: (line: TimelineCaption) => void;
  /** Told stretches moved, in the order they play, to play together
   * before a recorded moment, or at the end for null. */
  onMove: (spans: Span[], before: number | null) => void;
  /** Told a jump pressed, by the recorded moment it names, to put what was
   * moved there back where it was recorded. */
  onPutBack: (at: number) => void;
  onSceneDraw: (from: number, to: number) => void;
  onSceneRetime: (id: string, start: number, end: number) => void;
  /** Told what is chosen, or null when the choice is let go. */
  onSelect: (selection: StripSelection | null) => void;
  onSqueezeChange: (squeeze: number) => void;
  /** Where the playhead is, in the edited recording. */
  playhead: Playhead;
  /** How long the recording was recorded, in seconds. */
  recorded: number;
  scenes: TimelineScene[];
  /** How many seconds the lulls are squeezed by now. */
  squeeze: number;
};

export function TimelineStrip(props: TimelineStripProps) {
  const { map, selection } = props;
  const frame = useWidth();
  const share = (seconds: number, isEnd?: boolean) =>
    map.toPlayed(seconds, isEnd) / Math.max(map.duration, 1e-6);
  const chosen = selectionSpan(selection, {
    annotation: props.annotations.map((each) => ({ id: each.id, span: each })),
    beat: props.beats,
    lull: props.lulls,
    scene: props.scenes.map((each) => ({ id: each.id, span: each })),
  });
  const view = useStripView({
    duration: map.duration,
    focus:
      chosen && selection
        ? {
            end: share(chosen.end, true),
            key: JSON.stringify(selection),
            start: share(chosen.start),
          }
        : null,
    isPlaying: props.isPlaying,
    playhead: props.playhead,
    width: frame.width,
  });
  const scale = timelineScale(map);
  const width = view.trackWidth;
  const scrub = useStripScrub({
    onDeselect: () => {
      if (selection) props.onSelect(null);
    },
    onSeek: props.onSeek,
    scale,
  });
  const range = useRangeCut({
    fractionOf: scrub.fractionOf,
    isCutting: props.isAltHeld,
    onChoose: (span) => {
      props.onSelect({ kind: "range", ...span });
    },
    secondsAt: scrub.secondsAt,
  });
  const reorder = useStripReorder({
    beats: props.beats.map((beat) => beat.span),
    cuts: props.cuts,
    fractionOf: scrub.fractionOf,
    map,
    onMove: props.onMove,
    width,
  });
  // The parts are drawn as the strip shows the recording, which a drag
  // lays out as if it were already let go.
  const shown = timelineScale(reorder.map);
  const drawn = splitParts(props, reorder.edges);
  const snap = beatEdgeSnap({
    beats: props.beats.map(({ span }) => span),
    scale,
    width,
  });
  const chosenId = (kind: "annotation" | "scene") =>
    selection?.kind === kind ? selection.id : undefined;

  return (
    <div
      className="relative flex min-w-0 grow flex-col gap-2 pe-5"
      style={
        { "--glide": motionDurationCss("rearrange") } as React.CSSProperties
      }
    >
      <div className="relative" dir="ltr" ref={frame.ref}>
        <ScrollArea
          className="px-2"
          dir="ltr"
          onInitialized={view.onInitialized}
          orientation="horizontal"
          scrollbarHidden
        >
          <div
            // The playhead's knob has a band of its own along the top,
            // above the scenes, so it is never taken for part of one.
            className={cn(
              "relative flex touch-none flex-col pt-3 pb-1",
              range.bladeX !== null && "cursor-col-resize",
              reorder.isGliding && GLIDE,
            )}
            onClickCapture={range.handlers.onClickCapture}
            onPointerDownCapture={range.handlers.onPointerDownCapture}
            onPointerLeave={range.handlers.onPointerLeave}
            onPointerMoveCapture={range.handlers.onPointerMoveCapture}
            onPointerUpCapture={range.handlers.onPointerUpCapture}
            ref={scrub.trackRef}
            style={{ width }}
            {...scrub.handlers}
          >
            <SceneRail
              duration={props.recorded}
              onDraw={props.onSceneDraw}
              onRetime={props.onSceneRetime}
              onSelect={(id) => {
                props.onSelect({ id, kind: "scene" });
              }}
              scale={shown}
              scenes={drawn.scenes}
              selected={chosenId("scene")}
              width={width}
            />
            <div className="mt-1">
              <AnnotationLane
                annotations={drawn.annotations}
                onRestack={props.onAnnotationRestack}
                onRetime={props.onAnnotationRetime}
                onSelect={(id) => {
                  props.onSelect({ id, kind: "annotation" });
                }}
                scale={shown}
                selected={chosenId("annotation")}
                snap={snap}
                width={width}
              />
            </div>
            {/* A spring's badge, on the top edge of its box, reaches a few
                px above the beats. */}
            <div className="relative mt-1" ref={reorder.rowRef}>
              <StripBeatRow
                {...drawn}
                draft={range.draft}
                lifted={reorder.lifted}
                onGrab={reorder.grab}
                onSelect={props.onSelect}
                scale={shown}
                width={width}
              />
              <JumpMarks
                jumps={props.jumps}
                onPutBack={props.onPutBack}
                scale={shown}
              />
            </div>
            {drawn.captions ? (
              <div className="mt-1">
                <CaptionLane
                  isHidden={props.isCaptionsHidden}
                  lines={drawn.captions}
                  // The playhead goes to the line, so the caption is on the
                  // picture.
                  onPress={(line) => {
                    props.onSeek(line.start);
                    props.onCaptionPress(line);
                  }}
                  scale={shown}
                  width={width}
                />
              </div>
            ) : null}
            {/* Over the scenes, which its line crosses, but pressed only by
                its knob. */}
            <span className="pointer-events-none absolute inset-x-0 top-1.5 bottom-0 z-20">
              <TimelinePlayhead
                onScrub={scrub.scrub}
                playhead={props.playhead}
              />
            </span>
            {range.bladeX === null ? null : <CutBlade x={range.bladeX} />}
          </div>
        </ScrollArea>
      </div>
      <div className={cn("px-2", reorder.isGliding && GLIDE)}>
        <Minimap
          beats={drawn.beats.map(({ beat, span }) => ({
            ...span,
            color: beat.kind === "moment" ? beat.color : undefined,
          }))}
          map={reorder.map}
          maxSqueeze={props.maxSqueeze}
          onSqueezeChange={props.onSqueezeChange}
          onViewChange={view.set}
          playhead={props.playhead}
          recorded={props.recorded}
          squeeze={props.squeeze}
          view={view.view}
        />
      </div>
    </div>
  );
}
