// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { recordingTranscriptCorrectionsMs } from "../../recording/transcript/recording-timeline-transcript";
import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { type EditorTransportProps } from "../editor-transport";
import { deletedRecordingKeyboardShortcutIds } from "../keyboard/recording-keyboard-timeline-edit";
import { type RecordingMoment } from "../moments/recording-moments";
import { isGapSegment } from "../recording-timeline-types";
import { type Playhead } from "../scrub-playhead";
import { type StripSelection } from "../strip/strip-selection";
import { type TimelineStripProps } from "../strip/timeline-strip-props";
import { useAltKey } from "../strip/use-alt-key";
import { type ScrubPhase } from "../timeline-seek";

import { correctedLines } from "./corrected-words";
import {
  recordingBeats,
  speechBeatId,
  withoutScreen,
  withoutScreenUnderBlank,
} from "./recording-beats";
import { movementsSeen } from "./recording-camera-beats";
import { recordingStripGapActions } from "./recording-strip-gap-actions";
import { recordingStripLulls } from "./recording-strip-lulls";
import { shownStripBeats, shownStripParts } from "./recording-strip-shown";
import { recordingStripTime } from "./recording-strip-time";
import {
  chosenSpans,
  partSpans,
  useRecordingStripEdits,
} from "./use-recording-strip-edits";
import { useRecordingStripReadings } from "./use-recording-strip-readings";
import { useStripTools } from "./use-strip-tools";

import type { RecordingTimelineThumbnail } from "../../types";

// The strip over the recording open in the editor: its beats and lulls
// found in what it holds, drawn as its timeline edit plays them, with the
// playhead and play of the native player, and every change made on it made
// to the edit. What is chosen on it is kept here, for the islands to show.

/** What the band under the picture needs of the editor. */
export type RecordingStripSource = {
  artifactId: number;
  /** How long the recording was recorded, in ms. */
  durationMs: number;
  edit: RecordingTimelineEdit;
  hasKeyboardData: boolean;
  /** Whether the camera is drawn into the video: saved as its own file or
   * out of the export, where it moves makes no beats. */
  isCameraShown: boolean;
  /** Whether the shortcuts are drawn over the picture. */
  isKeyboardShown: boolean;
  isPlaying: boolean;
  /** Whether the screen's track is in the export: out, the strip is made
   * of what is left, the voice and the moments. */
  isScreenShown: boolean;
  moments: RecordingMoment[];
  onEdit: (next: RecordingTimelineEdit) => void;
  onPlayingChange: (isPlaying: boolean) => void;
  pictureAspect: number;
  playhead: Playhead;
  /** Moves the playhead to a share of the edited recording. */
  seek: (ratio: number, phase: ScrubPhase) => void;
  /** Frames along the recording, evenly, as they are read. */
  thumbnails: RecordingTimelineThumbnail[];
};

/** The strip's parts that the lanes fill, and what they are told. */
export type RecordingStripLanes = Pick<
  TimelineStripProps,
  | "annotations"
  | "onAnnotationRestack"
  | "onAnnotationRetime"
  | "onCaptionPress"
  | "onSceneDraw"
  | "onSceneRetime"
  | "scenes"
>;

export function useRecordingStrip(source: RecordingStripSource) {
  const { artifactId, durationMs, edit } = source;
  const recorded = durationMs / 1000;
  const readings = useRecordingStripReadings(source);
  const { isCameraShown, isScreenShown } = source;
  const time = useMemo(
    () => recordingStripTime(edit, recorded),
    [edit, recorded],
  );
  const found = useMemo(() => {
    const screen = isScreenShown
      ? withoutScreenUnderBlank(readings, edit.sceneClips, time.map)
      : withoutScreen(readings);
    return recordingBeats({
      ...screen,
      camera: isCameraShown
        ? movementsSeen(readings.camera, edit.sceneClips, time.map)
        : [],
    });
  }, [edit.sceneClips, isCameraShown, isScreenShown, readings, time.map]);
  const lulls = useMemo(
    () =>
      recordingStripLulls({
        cuts: time.cuts,
        edit,
        found: found.lulls,
        recorded,
      }),
    [edit, found.lulls, recorded, time.cuts],
  );
  const [chosen, setChosen] = useState<{
    artifactId: number;
    selection: StripSelection | null;
  } | null>(null);
  const selection = chosen?.artifactId === artifactId ? chosen.selection : null;
  const select = (next: StripSelection | null) => {
    setChosen({ artifactId, selection: next });
  };
  const tools = useStripTools({
    map: time.map,
    playhead: source.playhead,
    select,
    selection,
  });
  const isAltHeld = useAltKey();
  const spanOf = partSpans({ beats: found.beats, lulls: lulls.lulls }, time);
  const edits = useRecordingStripEdits({
    edit,
    lulls,
    onEdit: source.onEdit,
    recorded,
    spanOf,
    time,
  });
  const deleted = new Set(
    deletedRecordingKeyboardShortcutIds(edit).map(String),
  );
  const { thumbnails } = source;
  // What is said reads as the transcript is corrected, as the captions and
  // the script read it.
  const lines = useMemo(
    () =>
      correctedLines(
        readings.words,
        recordingTranscriptCorrectionsMs(edit, durationMs),
      ),
    [durationMs, edit, readings.words],
  );
  const said = new Map(lines.map((line) => [speechBeatId(line.start), line]));
  const beats = shownStripBeats({
    deletedShortcuts: deleted,
    found: found.beats,
    gaps: edit.segments.filter(isGapSegment),
    isKeyboardShown: source.isKeyboardShown,
    isScreenShown,
    recorded,
    said,
    thumbnails,
  });
  const seekTo = (seconds: number, phase?: ScrubPhase) => {
    // A gap's moments lie past the recording's end, where the map has them.
    const ratio =
      time.map.toPlayed(Math.max(seconds, 0)) /
      Math.max(time.map.duration, 1e-6);
    if (phase) {
      source.seek(ratio, phase);
      return;
    }
    source.seek(ratio, "start");
    source.seek(ratio, "end");
  };

  const strip = (lanes: RecordingStripLanes): TimelineStripProps =>
    shownStripParts(stripProps(lanes), time);
  const stripProps = (lanes: RecordingStripLanes): TimelineStripProps => ({
    ...lanes,
    beats,
    captions: lines.length > 0 ? lines : null,
    cuts: time.cuts,
    gapped: time.gapped,
    isAltHeld,
    isCaptionsHidden: edit.captions?.enabled !== true,
    ...tools.strip,
    isPlaying: source.isPlaying,
    isScreenShown,
    jumps: time.jumps,
    lulls: lulls.lulls,
    map: time.map,
    maxSqueeze: lulls.maxSqueeze,
    onBeatTrim: edits.onBeatTrim,
    onDeleteShortcuts: (ids) => {
      edits.steps.once({
        ...edit,
        deletedKeyboardShortcutIds: [
          ...new Set([
            ...deletedRecordingKeyboardShortcutIds(edit),
            ...ids.map(Number),
          ]),
        ].sort((one, other) => one - other),
      });
    },
    onGapSlide: edits.onGapSlide,
    onLullPlayChange: edits.onLullPlayChange,
    onLullTrim: edits.onLullTrim,
    onMove: edits.onMove,
    onPutBack: edits.onPutBack,
    onRemoveSplit: edits.onRemoveSplit,
    onRestore: edits.onRestore,
    onSeek: seekTo,
    onSelect: select,
    onSplit: edits.onSplit,
    onSqueezeChange: edits.onSqueezeChange,
    pictureAspect: source.pictureAspect,
    playhead: source.playhead,
    recorded,
    runs: time.runs,
    selection,
    // Read off the screen, so they go with it.
    shortcuts: (isScreenShown ? readings.shortcuts : [])
      .filter(({ id }) => !deleted.has(id))
      .map(({ at, end, id, keys }) => ({ end, id, keys, start: at })),
    splits: time.splits,
    squeeze: lulls.squeeze,
  });
  const gaps = recordingStripGapActions({
    durationMs,
    edit,
    map: time.map,
    playhead: source.playhead,
    select,
    selection,
    steps: edits.steps,
  });
  // The strip's half of the transport: its play, its blade, its range and
  // a gap inserted at the playhead.
  const transport: Pick<
    EditorTransportProps,
    "durationMs" | "isPlaying" | "onInsertGap" | "onPlayingChange" | "playhead"
  > &
    typeof tools.transport = {
    ...tools.transport,
    durationMs: time.map.duration * 1000,
    isPlaying: source.isPlaying,
    onInsertGap: gaps.insert,
    onPlayingChange: source.onPlayingChange,
    playhead: source.playhead,
  };
  const chosenStretches = chosenSpans(selection, spanOf);
  // A gap chosen is taken away and closed up, its songs and sound effects
  // moving back with what came after it; anything else is cut.
  const cutChosen = () => {
    if (gaps.chosenId !== null) {
      gaps.remove();
      return;
    }
    if (chosenStretches.length === 0) return;
    edits.cut(chosenStretches);
    select(null);
  };
  // Delete takes a chosen beat, lull, gap or range and its time; the lanes'
  // own choices are deleted by theirs.
  const deselect = selection
    ? () => {
        select(null);
      }
    : undefined;
  useEditorWindowShortcuts({
    onDelete: chosenStretches.length > 0 ? cutChosen : undefined,
    onDeselect: deselect,
    onDeselectAll: deselect,
    onInsertGap: gaps.insert,
  });
  return {
    beats,
    /** Cuts what is chosen, a beat, a lull, several or a drawn range. */
    cutChosen,
    /** Gives the gap chosen on the strip a background, or the recording's
     * again for null. */
    dressGap: gaps.dress,
    lulls,
    /** Brings the playhead to a moment, in recorded seconds. */
    seekTo,
    select,
    selection,
    steps: edits.steps,
    strip,
    time,
    transport,
  };
}
