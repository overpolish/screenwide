// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { type Mark } from "../../ideas/look-pens-model";
import { type Pen } from "../../ideas/looks";
import { type Backdrop, MARK_S, type ScenePreset } from "../editor-data";
import { type Recording } from "../recording";

import { type AnnotationStyle } from "./annotation-style";
import { beatsOf } from "./holds";
import {
  matchedTo,
  retimed,
  seededMarks,
  timedMark,
  type TimedMark,
} from "./marks";
import { useHistory } from "./use-history";
import { useOverlays } from "./use-overlays";
import { usePenStyles } from "./use-pen-styles";
import { usePlayback } from "./use-playback";
import { useScenes } from "./use-scenes";
import { useTiming } from "./use-timing";

// Everything the mock editor remembers, and the few things it does with it.
// The island is not stored: it follows from the tool in hand,
// the selection and the mode opened, so it can never disagree with them.

export type EditorKind = "audio" | "recording" | "screenshot";
export type Tool = Pen | "select";
export type Selection =
  | { id: number; kind: "mark" }
  | { id: string; kind: "beat" }
  | { id: string; kind: "scene" }
  | { id: string; kind: "pause" }
  /** A stretch drawn across the strip with ⌥ held, to cut. */
  | { end: number; kind: "range"; start: number }
  | null;
export type Mode =
  "captions" | "cursor" | "frame" | "keyboard" | "scene" | null;
/** What is drawn over the whole recording, each shown or hidden. */
export type Effects = { captions: boolean; keys: boolean; ripple: boolean };

export type EditorStart = {
  mode?: Mode;
  playhead?: number;
  selection?: Selection;
};

export function useEditor({
  isVoiced,
  kind,
  recording,
  start = {},
}: {
  isVoiced: boolean;
  kind: EditorKind;
  recording: Recording;
  start?: EditorStart;
}) {
  const [playhead, setPlayhead] = useState(start.playhead ?? 20);
  const [isPlaying, setIsPlaying] = useState(false);
  const [selection, setSelection] = useState<Selection>(
    start.selection ?? null,
  );
  const [tool, setTool] = useState<Tool>("select");
  // The scissors on the strip, on until turned off or stepped back from.
  // Turned off, they take the stretch they drew with them.
  const [isCutting, setIsCutting] = useState(false);
  const setCutting = (on: boolean) => {
    setIsCutting(on);
    if (!on)
      setSelection((current) => (current?.kind === "range" ? null : current));
  };
  const [mode, setMode] = useState<Mode>(start.mode ?? null);
  // Opened from something on the strip, such as a caption, a mode's island
  // stands over it rather than by the toolbar.
  const [modeFrom, setModeFrom] = useState<{ end: number; start: number }>();
  const [marks, setMarks] = useState<TimedMark[]>(() => seededMarks(recording));
  const sceneClips = useScenes(recording);
  const styles = usePenStyles();
  const overlays = useOverlays();
  const [backdrop, setBackdrop] = useState<Backdrop>("none");
  const [effects, setEffects] = useState<Effects>({
    captions: isVoiced,
    keys: true,
    ripple: true,
  });

  const beats = useMemo(
    () => beatsOf(recording, { isVoiced, kind }),
    [isVoiced, kind, recording],
  );
  const timing = useTiming({ beats, isVoiced, recording });
  const { map } = timing;
  // What undo takes back: the timing, the annotations and the scenes. Not
  // the selection, the playhead or the tool, which are where the user is,
  // not what they made.
  const doc = useMemo(
    () => ({
      cuts: timing.cuts,
      marks,
      scenes: sceneClips.scenes,
      settings: timing.settings,
      squeeze: timing.squeeze,
    }),
    [marks, sceneClips.scenes, timing.cuts, timing.settings, timing.squeeze],
  );
  const history = useHistory(doc, (past) => {
    timing.restoreTiming(past);
    setMarks(past.marks);
    sceneClips.setScenes(past.scenes);
  });

  usePlayback({
    isPlaying,
    map,
    onEnd: () => {
      setIsPlaying(false);
    },
    onPlayhead: setPlayhead,
    playhead,
  });

  const seek = (seconds: number) => {
    setPlayhead(Math.min(Math.max(seconds, 0), recording.duration - 0.01));
  };

  const togglePlay = () => {
    if (!isPlaying && map.toTime(playhead) >= map.duration - 0.05) seek(0);
    setIsPlaying((current) => !current);
  };

  const select = (next: Selection) => {
    setSelection(next);
    setMode(null);
    if (next?.kind === "mark") {
      const mark = marks.find(({ id }) => id === next.id);
      if (mark && (playhead < mark.start || playhead >= mark.end))
        seek(mark.start);
    }
    // A scene chosen is shown from its start, so it is seen as it plays.
    if (next?.kind === "scene") {
      const clip = sceneClips.scenes.find(({ id }) => id === next.id);
      if (clip && (playhead < clip.start || playhead >= clip.end))
        seek(clip.start);
    }
  };

  const pickTool = (next: Tool) => {
    setTool(next);
    setMode(null);
    setSelection(null);
  };

  // A new mark shows from the moment its element was used, so placing it
  // moves the playhead there: the mark stays in view, timed by the action.
  const place = (mark: Mark) => {
    // The use of its element nearest the playhead: a long recording uses
    // the same control many times.
    const action = recording.actions
      .filter(({ element }) => element === mark.element.id)
      .reduce<Recording["actions"][number] | undefined>(
        (best, item) =>
          !best || Math.abs(item.at - playhead) < Math.abs(best.at - playhead)
            ? item
            : best,
        undefined,
      );
    const from = action ? action.at : playhead;
    const timed = timedMark(
      mark,
      { end: Math.min(recording.duration, from + MARK_S), start: from },
      styles.penStyles[mark.pen],
    );
    setMarks((current) => [...current, timed]);
    setTool("select");
    setSelection({ id: mark.id, kind: "mark" });
    seek(from);
  };

  /** Restyles the chosen annotation, and it alone. */
  const updateMark = (change: Partial<AnnotationStyle>) => {
    if (selection?.kind !== "mark") return;
    setMarks((current) =>
      current.map((mark) =>
        mark.id === selection.id
          ? { ...mark, style: { ...mark.style, ...change } }
          : mark,
      ),
    );
  };

  const remove = () => {
    if (!selection) return;
    if (selection.kind === "mark")
      setMarks((current) => current.filter(({ id }) => id !== selection.id));
    if (selection.kind === "beat") {
      const beat = beats.find(({ id }) => id === selection.id);
      if (beat) timing.cut({ end: beat.end, id: beat.id, start: beat.start });
    }
    if (selection.kind === "scene") sceneClips.removeScene(selection.id);
    if (selection.kind === "pause") {
      const pause = map.pauses.find(({ id }) => id === selection.id);
      if (pause)
        timing.cut({ end: pause.end, id: pause.id, start: pause.start });
    }
    if (selection.kind === "range") {
      timing.cutRange(selection.start, selection.end);
    }
    setSelection(null);
  };

  /** Steps back one level: the scissors, the pen, then a mode, then the
   * selection. */
  const back = () => {
    if (isCutting) setCutting(false);
    else if (tool !== "select") setTool("select");
    else if (mode) setMode(null);
    else setSelection(null);
  };
  return {
    ...styles,
    ...timing,
    ...overlays,
    /** Adds a scene at the playhead, or gives the one there `preset`, and
     * chooses it. */
    addScene: (preset: ScenePreset) => {
      const id = sceneClips.addScene(preset, playhead, beats);
      setSelection({ id, kind: "scene" });
      setMode(null);
    },
    back,
    backdrop,
    beats,
    canRedo: history.canRedo,
    canUndo: history.canUndo,
    /** Adds a zoom drawn along the scene rail, and chooses it. */
    drawScene: (from: number, to: number) => {
      const id = sceneClips.drawScene(from, to);
      if (id) setSelection({ id, kind: "scene" });
    },
    effects,
    isCutting,
    isPlaying,
    isVoiced,
    kind,
    map,
    marks,
    /** Draws every annotation of the chosen one's kind as it is drawn. */
    matchAll: () => {
      const chosen = marks.find(
        ({ id }) => selection?.kind === "mark" && id === selection.id,
      );
      if (chosen) setMarks((current) => matchedTo(current, chosen));
    },
    mode,
    modeFrom,
    pickTool,
    place,
    playhead,
    recording,
    redo: history.redo,
    remove,
    retimeMark: (id: number, start: number, end: number) => {
      setMarks((current) =>
        retimed(current, { duration: recording.duration, end, id, start }),
      );
    },
    retimeScene: sceneClips.retimeScene,
    scenes: sceneClips.scenes,
    seek,
    select,
    selection,
    setBackdrop,
    setCutting,
    setEffects,
    /** Opens a mode, from the toolbar or, given `from`, from that stretch
     * of the strip. */
    setMode: (next: Mode, from?: { end: number; start: number }) => {
      setMode(next);
      setModeFrom(from);
      setSelection(null);
      setTool("select");
    },
    togglePlay,
    tool,
    undo: history.undo,
    updateMark,
    updateScene: sceneClips.updateScene,
  };
}

export type Editor = ReturnType<typeof useEditor>;
