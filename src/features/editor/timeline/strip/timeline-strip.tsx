// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { motionDurationCss } from "../../../../lib/motion";
import { cn } from "../../../../lib/styling";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { AnnotationLane } from "../annotation-lane/annotation-lane";
import { AudioLanes } from "../audio-lane/audio-lanes";
import { CaptionLane } from "../caption-lane/caption-lane";
import { Minimap } from "../minimap/minimap";
import { CutBlade } from "../range-cut/cut-blade";
import { RangeBand } from "../range-cut/range-band";
import { SceneRail } from "../scene-rail/scene-rail";
import { ShortcutLane } from "../shortcut-lane/shortcut-lane";
import { at, timelineScale } from "../timeline-scale";

import { SnapGuide } from "./snap-guide";
import { StripBeats } from "./strip-beats";
import { stripChoice } from "./strip-choice";
import { GLIDE, GlideCue } from "./strip-glide";
import { StripLane } from "./strip-lane";
import { StripScrubBand } from "./strip-scrub-band";
import { stripSnap } from "./strip-snap";
import { splitParts } from "./strip-split";
import { takenOut, takenTotal } from "./taken-out";
import { type TimelineStripProps } from "./timeline-strip-props";
import { useRangeCut } from "./use-range-cut";
import { useStripReorder } from "./use-strip-reorder";
import { useStripReshape } from "./use-strip-reshape";
import { useStripScrub } from "./use-strip-scrub";
import { useStripView } from "./use-strip-view";
import { useWidth } from "./use-width";

// The timeline as one strip of what happened, to scale, showing the part of
// the recording in view; the minimap under it shows the whole. Along its
// top are the scenes, under them the annotations and the shortcuts pressed,
// then what was done, each beat as the screen at that moment with the
// lulls between as springs, under the beats the voice's captions, and
// under them the songs and sound effects. The playhead crosses them all;
// past the end, a band accounts for what was taken out, so the strip stays
// as long as the recording, then the squeeze grip.
//
// Pressing a part selects it, and moves the playhead to its start only
// where the playhead is outside it, so the picture shows what was chosen;
// pressing the empty strip moves the playhead there; dragging the playhead's
// knob scrubs.
// With the blade in hand, from its tool or ⌥ held, a press splits the
// recording there; with the range tool, a drag draws a range. A beat, a
// lull, what is chosen or the range, dragged along the strip, moves
// to play elsewhere, the rest making room for it as it goes.

export function TimelineStrip(props: TimelineStripProps) {
  const { map, selection } = props;
  const frame = useWidth();
  /** Where a scene's or song's end has snapped while drawn or dragged, if
   * anywhere. */
  const [guide, setGuide] = useState<number | null>(null);
  const share = (seconds: number, isEnd?: boolean) =>
    map.toPlayed(seconds, isEnd) / Math.max(map.duration, 1e-6);
  const { choose, chosen, chosenId } = stripChoice(props);
  const taken = takenOut(props.cuts, map.duration, props.recorded);
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
    takenOut: takenTotal(taken),
    width: frame.width,
  });
  const scale = timelineScale(map);
  const width = view.trackWidth;
  const scrub = useStripScrub({
    onDeselect: () => {
      if (selection) choose(null);
    },
    onSeek: props.onSeek,
    scale,
  });
  const range = useRangeCut({
    onChoose: (span) => {
      choose({ kind: "range", ...span });
    },
    scrub,
    strip: props,
  });
  const shownRange =
    range.draft ?? (selection?.kind === "range" ? selection : null);
  const reorder = useStripReorder({
    beats: props.beats.map((beat) => beat.span),
    cuts: props.cuts,
    fractionOf: scrub.fractionOf,
    map,
    onMove: props.onMove,
    splits: props.splits,
    width,
  });
  // The parts are drawn as the strip shows the recording, which a drag
  // lays out as if it were already let go.
  const shown = timelineScale(reorder.map);
  const drawn = splitParts(props, reorder.edges);
  const reshape = useStripReshape({
    ...drawn,
    scale: shown,
    taken,
    width,
  });
  const isGliding = reorder.isGliding || reshape.isReshaping;
  // With ⌘ held, a scene or an annotation's bar dragged settles on one list
  // of edges (`strip-snap.ts`).
  const snap = stripSnap({
    parts: props,
    playheadAt: () => map.toRecorded(props.playhead.ratio() * map.duration),
    scale,
    width,
  });
  const chosenShortcuts = selection?.kind === "shortcuts" ? selection.ids : [];

  return (
    <div
      className="relative flex min-w-0 grow flex-col gap-2 pe-1"
      style={
        { "--glide": motionDurationCss("rearrange") } as React.CSSProperties
      }
    >
      <GlideCue isGliding={isGliding} />
      <div className={cn("relative", GLIDE)} dir="ltr" ref={frame.ref}>
        <ScrollArea
          className="px-2"
          dir="ltr"
          onInitialized={view.onInitialized}
          orientation="horizontal"
          scrollbarHidden
        >
          <div style={{ width: width + view.bandWidth }}>
            <div
              className={cn(
                "relative flex touch-none flex-col pt-8 pb-1",
                range.bladeX !== null && "cursor-col-resize",
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
              <StripScrubBand onScrub={scrub.scrub} playhead={props.playhead} />
              {/* Locked, a press falls through the lanes and scrubs. */}
              <div className="contents" inert={props.isLocked}>
                <SceneRail
                  duration={props.recorded}
                  onDraw={props.onSceneDraw}
                  onGuide={setGuide}
                  onRetime={props.onSceneRetime}
                  onSelect={(id) => {
                    choose({ id, kind: "scene" });
                  }}
                  scale={shown}
                  scenes={drawn.scenes}
                  selected={chosenId("scene")}
                  snap={snap}
                  width={width}
                />
                <StripLane>
                  {drawn.annotations.length > 0 ? (
                    <AnnotationLane
                      annotations={drawn.annotations}
                      onRestack={props.onAnnotationRestack}
                      onRetime={props.onAnnotationRetime}
                      onSelect={(id) => {
                        choose({ id, kind: "annotation" });
                      }}
                      scale={shown}
                      selected={chosenId("annotation")}
                      snap={snap}
                      width={width}
                    />
                  ) : null}
                </StripLane>
                <StripLane>
                  {drawn.shortcuts.length > 0 ? (
                    <ShortcutLane
                      onDelete={(ids) => {
                        props.onDeleteShortcuts(ids);
                        choose(null);
                      }}
                      onSelect={(ids) => {
                        choose(
                          ids.length > 0 ? { ids, kind: "shortcuts" } : null,
                        );
                      }}
                      scale={shown}
                      selected={chosenShortcuts}
                      shortcuts={drawn.shortcuts}
                      width={width}
                    />
                  ) : null}
                </StripLane>
                <StripBeats
                  {...drawn}
                  lifted={reorder.lifted}
                  onGrab={reorder.grab}
                  onPutBack={props.onPutBack}
                  onRemoveSplit={props.onRemoveSplit}
                  onSelect={choose}
                  pxPerSecond={view.pxPerSecond}
                  reshape={reshape}
                  rowRef={reorder.rowRef}
                  scale={shown}
                  taken={taken}
                  width={width}
                />
                <StripLane>
                  {drawn.captions ? (
                    <CaptionLane
                      isHidden={props.isCaptionsHidden}
                      lines={drawn.captions}
                      // The playhead goes to the line, to show its caption.
                      onPress={(line) => {
                        props.onSeek(line.start);
                        props.onCaptionPress(line);
                      }}
                      scale={shown}
                      width={width}
                    />
                  ) : null}
                </StripLane>
                <AudioLanes
                  audio={props.audio}
                  onGuide={setGuide}
                  onSelect={(id) => {
                    choose({ id, kind: "audioClip" });
                  }}
                  scale={shown}
                  selected={chosenId("audioClip")}
                  snap={snap}
                  width={width}
                />
              </div>
              {/* Over every lane; carried, its row draws it (`RangeBand`). */}
              {shownRange && reorder.lifted === null ? (
                <RangeBand
                  className="top-3.5 bottom-1"
                  scale={shown}
                  span={shownRange}
                />
              ) : null}
              {guide === null ? null : (
                <SnapGuide
                  className="top-8 bottom-0"
                  style={at(shown, guide)}
                />
              )}
              {range.bladeX === null ? null : <CutBlade x={range.bladeX} />}
            </div>
          </div>
        </ScrollArea>
      </div>
      <div className={cn("px-2", GLIDE)} inert={props.isLocked}>
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
          squeeze={props.squeeze}
          taken={taken}
          view={view.view}
        />
      </div>
    </div>
  );
}
