// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { recordingTranscriptCorrectionsMs } from "../../recording/transcript/recording-timeline-transcript";
import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { type EditorTransportProps } from "../editor-transport";
import { deletedRecordingKeyboardShortcutIds } from "../keyboard/recording-keyboard-timeline-edit";
import { type RecordingMoment } from "../moments/recording-moments";
import { type Playhead } from "../scrub-playhead";
import { type StripSelection } from "../strip/strip-selection";
import { SLIVER_S, splitParts } from "../strip/strip-split";
import { type TimelineStripProps } from "../strip/timeline-strip-props";
import { useAltKey } from "../strip/use-alt-key";
import { type Span } from "../timeline-scale";
import { type ScrubPhase } from "../timeline-seek";

import { correctedLines } from "./corrected-words";
import { recordingBeats, speechBeatId, withoutScreen } from "./recording-beats";
import { recordingStripLulls } from "./recording-strip-lulls";
import { isPlayed, recordingStripTime } from "./recording-strip-time";
import {
  chosenSpans,
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
  const { isScreenShown } = source;
  const found = useMemo(
    () => recordingBeats(isScreenShown ? readings : withoutScreen(readings)),
    [isScreenShown, readings],
  );
  const time = useMemo(
    () => recordingStripTime(edit, recorded),
    [edit, recorded],
  );
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
  const edits = useRecordingStripEdits({
    beats: found.beats,
    edit,
    lulls,
    onEdit: source.onEdit,
    recorded,
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
  const beats = found.beats.map((beat) => {
    const middle = (beat.span.start + beat.span.end) / 2;
    const frame =
      thumbnails.length > 0
        ? thumbnails[
            Math.min(
              thumbnails.length - 1,
              Math.floor(
                (middle / Math.max(recorded, 1e-6)) * thumbnails.length,
              ),
            )
          ]
        : undefined;
    const line = beat.beat.kind === "speech" ? said.get(beat.id) : undefined;
    return {
      ...beat,
      ...(line ? { beat: { kind: "speech", line: line.text } as const } : {}),
      isHidden:
        beat.beat.kind === "keys" &&
        (!source.isKeyboardShown || deleted.has(beat.id.slice("keys-".length))),
      // With the screen out there is no picture to show.
      thumbnail: isScreenShown ? (frame?.url ?? null) : undefined,
    };
  });
  const seekTo = (seconds: number, phase?: ScrubPhase) => {
    const ratio =
      time.map.toPlayed(Math.min(Math.max(seconds, 0), recorded)) /
      Math.max(time.map.duration, 1e-6);
    if (phase) {
      source.seek(ratio, phase);
      return;
    }
    source.seek(ratio, "start");
    source.seek(ratio, "end");
  };

  // Captions and shortcuts are read as recorded, so one a cut took out
  // whole would be drawn at the seam as a sliver, or as keys never pressed
  // in what plays (a beat leaves itself out under a px): those are left
  // out. What runs across a jump is drawn as a part each side of it; the
  // lanes' annotations and scenes come in those parts already, read from
  // where they play (`recordingStripLanes`). Where an item's edge and a
  // jump's are one moment apart only by rounding, a part would be a
  // sliver: those are left out too.
  const strip = (lanes: RecordingStripLanes): TimelineStripProps => {
    const props = stripProps(lanes);
    const made = {
      ...splitParts(props, time.edges),
      annotations: props.annotations,
      scenes: props.scenes,
    };
    const captions =
      made.captions?.filter((span) => isPlayed(time.map, span)) ?? [];
    const shown = {
      ...made,
      captions: captions.length > 0 ? captions : null,
      shortcuts: made.shortcuts.filter((span) => isPlayed(time.map, span)),
    };
    if (time.edges.length === 0) return shown;
    const isWhole = ({ end, start }: Span) => end - start >= SLIVER_S;
    return {
      ...shown,
      beats: shown.beats.filter(({ span }) => isWhole(span)),
      captions: shown.captions?.filter(isWhole) ?? null,
      cuts: shown.cuts.filter(isWhole),
      lulls: shown.lulls.filter(({ span }) => isWhole(span)),
      shortcuts: shown.shortcuts.filter(isWhole),
    };
  };
  const stripProps = (lanes: RecordingStripLanes): TimelineStripProps => ({
    ...lanes,
    beats,
    captions: lines.length > 0 ? lines : null,
    cuts: time.cuts,
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
    selection,
    // Read off the screen, so they go with it.
    shortcuts: (isScreenShown ? readings.shortcuts : [])
      .filter(({ id }) => !deleted.has(id))
      .map(({ at, end, id, keys }) => ({ end, id, keys, start: at })),
    splits: time.splits,
    squeeze: lulls.squeeze,
  });
  // The strip's half of the transport: its play, its blade and its range.
  const transport: Pick<
    EditorTransportProps,
    "durationMs" | "isPlaying" | "onPlayingChange" | "playhead"
  > &
    typeof tools.transport = {
    ...tools.transport,
    durationMs: time.map.duration * 1000,
    isPlaying: source.isPlaying,
    onPlayingChange: source.onPlayingChange,
    playhead: source.playhead,
  };
  const chosenStretches = chosenSpans(selection, { beats: found.beats, lulls });
  const cutChosen = () => {
    if (chosenStretches.length === 0) return;
    edits.cut(chosenStretches);
    select(null);
  };
  // Delete takes a chosen beat, lull or range and its time; the lanes'
  // own choices are deleted by theirs.
  useEditorWindowShortcuts({
    onDelete: chosenStretches.length > 0 ? cutChosen : undefined,
    onDeselect: selection
      ? () => {
          select(null);
        }
      : undefined,
    onDeselectAll: selection
      ? () => {
          select(null);
        }
      : undefined,
  });
  return {
    beats,
    /** Cuts what is chosen, a beat, a lull, several or a drawn range. */
    cutChosen,
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
