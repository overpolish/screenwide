// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { useRestingAction } from "../../../../storybook/use-resting-action";
import { createPlayhead } from "../scrub-playhead";
import { type Span } from "../timeline-scale";
import { storyFrameAspect } from "../timeline-story-fixtures";

import { type StripLull } from "./strip-beat-row";
import { type StripSelection } from "./strip-selection";
import { type LiveLull, useStoryContent } from "./strip-story-content";
import { type StoryRecording } from "./strip-story-fixtures";
import { jumpsOf, playedTime } from "./strip-story-order";
import { TimelineStrip, type TimelineStripProps } from "./timeline-strip";
import { useAltKey } from "./use-alt-key";

// The strip over a story recording, kept as the editor would keep it: what
// is chosen, the playhead, the squeeze, each lull's play and trim, the
// scenes, the annotations and the order it plays in. Drags are logged to
// Actions once they rest.
//
// A lull plays at one speed throughout. Squeezed, it is hurried by the same
// share of what it can give as every other: up to four times as fast, or
// twice where the picture moves by itself, since motion played faster
// cannot be followed.

/** How long, in seconds, a squeezed lull can still play and be dead air. */
const DEAD_AIR_S = 3;

export type LiveStripProps = Pick<
  TimelineStripProps,
  | "onAnnotationRestack"
  | "onAnnotationRetime"
  | "onCaptionPress"
  | "onLullPlayChange"
  | "onBeatTrim"
  | "onLullTrim"
  | "onMove"
  | "onPutBack"
  | "onRestore"
  | "onSceneDraw"
  | "onSceneRetime"
  | "onSeek"
  | "onSelect"
  | "onSqueezeChange"
> & {
  /** Whether ⌥ is held, besides the key itself. */
  isAltHeld: boolean;
  isCaptionsHidden: boolean;
  isPlaying: boolean;
  /** Stretches moved before the story opens, each to play before a
   * recorded moment, or at the end for null. */
  moves: { before: number | null; span: Span }[];
  recording: StoryRecording;
  selection: StripSelection | null;
  /** How far the lulls are squeezed, from 0 as recorded to 1 as far as they
   * go. */
  squeeze: number;
};

const fastest = (lull: LiveLull) => (lull.moving ? 2 : 4);
const lengthOf = ({ span }: LiveLull) => span.end - span.start;
const roomIn = (lull: LiveLull) =>
  lull.play.kind === "squeeze" ? lengthOf(lull) * (1 - 1 / fastest(lull)) : 0;

export function LiveStrip(props: LiveStripProps) {
  const { recording } = props;
  const edit = useStoryContent(recording, props.moves);
  const { content } = edit;
  const maxSqueeze = content.lulls.reduce((sum, lull) => sum + roomIn(lull), 0);
  const [squeeze, setSqueeze] = useState(props.squeeze * maxSqueeze);
  const [selection, setSelection] = useState(props.selection);
  const [playhead] = useState(createPlayhead);
  const altKey = useAltKey();
  const log = {
    annotation: useRestingAction(props.onAnnotationRetime),
    beatTrim: useRestingAction(props.onBeatTrim),
    lull: useRestingAction(props.onLullPlayChange),
    scene: useRestingAction(props.onSceneRetime),
    seek: useRestingAction(props.onSeek),
    squeeze: useRestingAction(props.onSqueezeChange),
    trim: useRestingAction(props.onLullTrim),
  };

  const share = maxSqueeze > 0 ? Math.min(squeeze / maxSqueeze, 1) : 0;
  const lulls: StripLull[] = content.lulls.map((lull) => {
    const length = lengthOf(lull);
    const plays =
      lull.play.kind === "keep"
        ? length
        : lull.play.kind === "retimed"
          ? length / lull.play.speed
          : length - share * roomIn(lull);
    return {
      ...lull,
      isDeadAir: lull.play.kind === "squeeze" && plays >= DEAD_AIR_S,
      plays,
    };
  });
  const map = playedTime(content.order, [
    ...lulls.map(({ plays, span }) => ({ ...span, plays })),
    ...content.cuts.map((cut) => ({ ...cut, plays: 0 })),
  ]);

  // Where the playhead is, in played seconds, kept through a squeeze or a
  // move by the recorded moment it stands on.
  const recordedRef = useRef(0);
  const publish = () => {
    const played = map.toPlayed(recordedRef.current);
    playhead.publish(played, played / map.duration);
  };
  useEffect(publish);
  useEffect(() => {
    if (!props.isPlaying) return;
    let frame = 0;
    let last = performance.now();
    const step = (now: number) => {
      const played = map.toPlayed(recordedRef.current) + (now - last) / 1000;
      last = now;
      recordedRef.current = map.toRecorded(played % map.duration);
      publish();
      frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => {
      cancelAnimationFrame(frame);
    };
  });

  return (
    <TimelineStrip
      {...content}
      isAltHeld={props.isAltHeld || altKey}
      isCaptionsHidden={props.isCaptionsHidden}
      isPlaying={props.isPlaying}
      jumps={jumpsOf(content.order)}
      lulls={lulls}
      map={map}
      maxSqueeze={maxSqueeze}
      onAnnotationRestack={(lanes) => {
        props.onAnnotationRestack(lanes);
        edit.restack(lanes);
      }}
      onAnnotationRetime={(id, start, end) => {
        log.annotation(id, start, end);
        edit.retimeAnnotation(id, start, end);
      }}
      onBeatTrim={(id, edge, at) => {
        log.beatTrim(id, edge, at);
        edit.trimBeat(id, edge, at);
      }}
      onCaptionPress={props.onCaptionPress}
      onLullPlayChange={(id, play) => {
        log.lull(id, play);
        edit.setPlay(id, play);
      }}
      onLullTrim={(id, end) => {
        log.trim(id, end);
        edit.trim(id, end);
      }}
      onMove={(spans, before) => {
        props.onMove(spans, before);
        edit.move(spans, before);
      }}
      onPutBack={(at) => {
        props.onPutBack(at);
        edit.putBack(at);
      }}
      onRestore={(ids) => {
        props.onRestore(ids);
        edit.restore(ids);
      }}
      onSceneDraw={(from, to) => {
        props.onSceneDraw(from, to);
        const drawn = edit.drawScene(from, to);
        if (drawn) setSelection({ id: drawn.id, kind: "scene" });
      }}
      onSceneRetime={(id, start, end) => {
        log.scene(id, start, end);
        edit.retimeScene(id, start, end);
      }}
      onSeek={(seconds) => {
        log.seek(seconds);
        recordedRef.current = Math.min(
          Math.max(seconds, 0),
          recording.duration,
        );
        publish();
      }}
      onSelect={(next) => {
        props.onSelect(next);
        setSelection(next);
      }}
      onSqueezeChange={(next) => {
        log.squeeze(next);
        setSqueeze(next);
      }}
      pictureAspect={storyFrameAspect()}
      playhead={playhead}
      recorded={recording.duration}
      selection={selection}
      squeeze={squeeze}
    />
  );
}
