// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BeatClip } from "../beat-clip/beat-clip";
import { type TimelineBeat } from "../beat-clip/beat-label";
import { type TimelineCaption } from "../caption-lane/caption-line";
import { CutSeams, type TimelineCut } from "../cut-seams/cut-seams";
import { RangeBand } from "../range-cut/range-band";
import { Spring } from "../spring/spring";
import { type Span, type TimelineScale, type TimeMap } from "../timeline-scale";

import { beatTrimOf } from "./strip-beat-trim";
import {
  chosenItems,
  isChosen,
  type StripItem,
  type StripSelection,
  toggled,
} from "./strip-selection";

import type { LullPlay } from "../../island/lull/lull-island";

// What was done, each beat drawn as the screen at that moment or as its
// sound, with the lulls between as springs and a seam wherever something
// was cut. The scissors' stretch is drawn over them all. A beat or lull
// pressed with Shift is taken into what is chosen, or left out if it was
// in; dragging any chosen one moves them all.

export type StripBeat = {
  beat: TimelineBeat;
  id: string;
  span: Span;
  /** Whether what it draws over the picture is hidden. */
  isHidden?: boolean;
  /** Its sound, as peaks from 0 to 1. */
  peaks?: number[];
  /** The screen at the beat, or null while it is read from the video. */
  thumbnail?: string | null;
};

export type StripLull = {
  id: string;
  /** Whether it is long enough still to be dead air. */
  isDeadAir: boolean;
  play: LullPlay;
  /** How long it plays now, in seconds. */
  plays: number;
  span: Span;
  /** Where the picture moves by itself in it. */
  moving?: Span[];
};

export type StripBeatRowProps = {
  beats: StripBeat[];
  /** The voice's lines, which a spoken beat's trim settles between. */
  captions: TimelineCaption[] | null;
  cuts: TimelineCut[];
  /** The stretch the scissors are drawing, while they draw it. */
  draft: Span | null;
  isAltHeld: boolean;
  /** What is lifted to be moved, while it is, in recorded seconds. */
  lifted: Span[] | null;
  lulls: StripLull[];
  /** How the recording plays now, which sets how wide a second is. */
  map: TimeMap;
  /** Told a beat's end trimmed to, or brought back to, a recorded moment.
   */
  onBeatTrim: (id: string, edge: "end" | "start", at: number) => void;
  /** Told a press on a beat, a lull or the chosen stretch, with the spans
   * that would move, which may go on to drag them to play elsewhere. */
  onGrab: (event: React.PointerEvent, spans: Span[]) => void;
  onLullPlayChange: (id: string, play: LullPlay) => void;
  /** Told a lull trimmed to end at a recorded moment. */
  onLullTrim: (id: string, end: number) => void;
  /** Told the cuts a seam puts back. */
  onRestore: (ids: string[]) => void;
  onSeek: (seconds: number) => void;
  onSelect: (selection: StripSelection | null) => void;
  pictureAspect: number;
  scale: TimelineScale;
  selection: StripSelection | null;
  /** The track's width, in px. */
  width: number;
};

export function StripBeatRow({
  beats,
  captions,
  cuts,
  draft,
  isAltHeld,
  lifted,
  lulls,
  map,
  onBeatTrim,
  onGrab,
  onLullPlayChange,
  onLullTrim,
  onRestore,
  onSeek,
  onSelect,
  pictureAspect,
  scale,
  selection,
  width,
}: StripBeatRowProps) {
  const chosenRange = selection?.kind === "range" ? selection : null;
  const spanOf = ({ id, kind }: StripItem) =>
    (kind === "beat" ? beats : lulls).find((each) => each.id === id)?.span;
  /** Chooses an item, or with Shift takes it in or leaves it out; true if
   * Shift was held. */
  const choose = (event: React.MouseEvent, item: StripItem) => {
    onSelect(event.shiftKey ? toggled(selection, item) : item);
    return event.shiftKey;
  };
  const isLifted = (span: Span) =>
    lifted?.some(
      (each) => span.start >= each.start - 1e-6 && span.end <= each.end + 1e-6,
    ) ?? false;
  // A chosen item takes the rest of what is chosen with it.
  const grab = (event: React.PointerEvent, item: StripItem, span: Span) => {
    onGrab(
      event,
      isChosen(selection, item.kind, item.id)
        ? chosenItems(selection).flatMap((each) => spanOf(each) ?? [])
        : [span],
    );
  };
  return (
    <div className="relative h-11">
      {lulls.map((lull) => (
        <Spring
          isAltHeld={isAltHeld}
          isDeadAir={lull.isDeadAir}
          isLifted={isLifted(lull.span)}
          isSelected={isChosen(selection, "lull", lull.id)}
          key={lull.id}
          moving={lull.moving}
          onGrab={(event) => {
            grab(event, { id: lull.id, kind: "lull" }, lull.span);
          }}
          onPlayChange={(play) => {
            onLullPlayChange(lull.id, play);
          }}
          onPress={(event) => {
            choose(event, { id: lull.id, kind: "lull" });
          }}
          onTrim={(end) => {
            onLullTrim(lull.id, end);
          }}
          play={lull.play}
          plays={lull.plays}
          scale={scale}
          span={lull.span}
          width={width}
        />
      ))}
      {beats.map((beat) => (
        <BeatClip
          beat={beat.beat}
          isHidden={beat.isHidden}
          isLifted={isLifted(beat.span)}
          isSelected={isChosen(selection, "beat", beat.id)}
          key={beat.id}
          onGrab={(event) => {
            grab(event, { id: beat.id, kind: "beat" }, beat.span);
          }}
          // The playhead goes to the beat, so the picture shows it; with
          // Shift, it stays where it is.
          onPress={(event) => {
            if (!choose(event, { id: beat.id, kind: "beat" }))
              onSeek(beat.span.start);
          }}
          peaks={beat.peaks}
          pictureAspect={pictureAspect}
          scale={scale}
          span={beat.span}
          thumbnail={beat.thumbnail}
          trim={beatTrimOf({
            beat,
            captions,
            cuts,
            map,
            onChoose: (id) => {
              onSelect({ id, kind: "beat" });
            },
            onTrim: onBeatTrim,
            width,
          })}
          width={width}
        />
      ))}
      <CutSeams cuts={cuts} onRestore={onRestore} scale={scale} />
      {draft ? (
        <RangeBand scale={scale} span={draft} />
      ) : chosenRange ? (
        <RangeBand
          isLifted={isLifted(chosenRange)}
          onGrab={(event) => {
            onGrab(event, [chosenRange]);
          }}
          scale={scale}
          span={chosenRange}
        />
      ) : null}
    </div>
  );
}
