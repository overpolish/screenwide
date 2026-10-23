// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useRef, useState } from "react";

import { type Mark, SEEDED } from "../../ideas/look-pens-model";
import { type LookId, type Pen } from "../../ideas/looks";
import { appearsAt } from "../../ideas/point-geometry";
import {
  ACTION_BEATS,
  DEFAULT_SCENES,
  type Layout,
  RECORDING_S,
  SEEDED_TIMES,
  SPEECH_BEATS,
  STEPS,
  stepAt,
  type Backdrop,
} from "../editor-data";

import { type Cut, timeMap } from "./timeline-model";

// Everything the mock editor remembers, and the few things it does with it.
// The panel in the corner is not stored: it follows from the tool in hand,
// the selection and the mode opened, so it can never disagree with them.

export type EditorKind = "audio" | "recording" | "screenshot";
export type Tool = Pen | "select";
export type Selection =
  | { id: number; kind: "mark" }
  | { id: string; kind: "beat" }
  | { index: number; kind: "step" }
  | null;
export type Mode = "captions" | "cursor" | "frame" | "look" | "scene" | null;
export type TimedMark = Mark & { end: number; start: number };
export type Effects = { captions: boolean; ripple: boolean };
export type View = "board" | "strip";

const SEEDED_MARKS: TimedMark[] = SEEDED.map((mark) => ({
  ...mark,
  ...SEEDED_TIMES[mark.id],
}));

export type EditorStart = {
  mode?: Mode;
  playhead?: number;
  selection?: Selection;
  view?: View;
};

export function useEditor({
  isVoiced,
  kind,
  start = {},
}: {
  isVoiced: boolean;
  kind: EditorKind;
  start?: EditorStart;
}) {
  const [playhead, setPlayhead] = useState(start.playhead ?? 20);
  const [isPlaying, setIsPlaying] = useState(false);
  const [selection, setSelection] = useState<Selection>(
    start.selection ?? null,
  );
  const [tool, setTool] = useState<Tool>("select");
  const [mode, setMode] = useState<Mode>(start.mode ?? null);
  const [isMoreOpen, setIsMoreOpen] = useState(false);
  const [marks, setMarks] = useState<TimedMark[]>(SEEDED_MARKS);
  const [cuts, setCuts] = useState<Cut[]>([]);
  const [squeeze, setSqueeze] = useState(0);
  const [scenes, setScenes] = useState<Layout[]>(DEFAULT_SCENES);
  const [view, setView] = useState<View>(start.view ?? "strip");
  const [openStep, setOpenStep] = useState<number | null>(null);
  const [lookId, setLookId] = useState<LookId>("product");
  const [backdrop, setBackdrop] = useState<Backdrop>("none");
  const [effects, setEffects] = useState<Effects>({
    captions: isVoiced,
    ripple: true,
  });

  const beats = useMemo(
    () =>
      kind === "audio"
        ? SPEECH_BEATS
        : kind === "recording"
          ? ACTION_BEATS
          : [],
    [kind],
  );
  const map = useMemo(
    () => timeMap({ beats, cuts, isVoiced, squeeze }),
    [beats, cuts, isVoiced, squeeze],
  );

  // Playing advances the edited time, so a squeezed pause passes quickly and
  // a cut is skipped, as the export will play it.
  const playheadRef = useRef(playhead);
  useEffect(() => {
    playheadRef.current = playhead;
  }, [playhead]);
  useEffect(() => {
    if (!isPlaying) return;
    let last = performance.now();
    let frame = 0;
    const tick = (now: number) => {
      const time = map.toTime(playheadRef.current) + (now - last) / 1000;
      last = now;
      if (time >= map.duration - 0.01) {
        setPlayhead(map.toOriginal(map.duration - 0.01));
        setIsPlaying(false);
        return;
      }
      playheadRef.current = map.toOriginal(time);
      setPlayhead(playheadRef.current);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isPlaying, map]);

  const seek = (seconds: number) => {
    setPlayhead(Math.min(Math.max(seconds, 0), RECORDING_S - 0.01));
  };

  const togglePlay = () => {
    if (!isPlaying && map.toTime(playhead) >= map.duration - 0.05) seek(0);
    setIsPlaying((current) => !current);
  };

  const select = (next: Selection) => {
    setSelection(next);
    setIsMoreOpen(false);
    setMode(null);
    if (next?.kind === "mark") {
      const mark = marks.find(({ id }) => id === next.id);
      if (mark && (playhead < mark.start || playhead >= mark.end))
        seek(mark.start);
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
    const action = appearsAt(mark.element);
    const from = action ? action.at : playhead;
    const step = STEPS[stepAt(from)];
    const timed = { ...mark, end: step.end, start: from };
    setMarks((current) => [...current, timed]);
    setTool("select");
    setSelection({ id: mark.id, kind: "mark" });
    seek(from);
  };

  const updateMark = (change: Partial<Mark>) => {
    if (selection?.kind !== "mark") return;
    setMarks((current) =>
      current.map((mark) =>
        mark.id === selection.id ? { ...mark, ...change } : mark,
      ),
    );
  };

  /** Moves when a mark shows, from the timeline; it keeps a sliver at
   * least, so a handle cannot be dragged past the other. */
  const retimeMark = (id: number, start: number, end: number) => {
    const from = Math.max(0, Math.min(start, end - 0.3));
    const to = Math.min(RECORDING_S, Math.max(end, from + 0.3));
    setMarks((current) =>
      current.map((mark) =>
        mark.id === id ? { ...mark, end: to, start: from } : mark,
      ),
    );
  };

  const cut = (next: Cut) => {
    setCuts((current) => [...current.filter(({ id }) => id !== next.id), next]);
  };

  const remove = () => {
    if (!selection) return;
    if (selection.kind === "mark")
      setMarks((current) => current.filter(({ id }) => id !== selection.id));
    if (selection.kind === "beat") {
      const beat = beats.find(({ id }) => id === selection.id);
      if (beat) cut({ end: beat.end, id: beat.id, start: beat.start });
    }
    if (selection.kind === "step") {
      const step = STEPS[selection.index];
      cut({
        end: step.end,
        id: `step-${String(selection.index)}`,
        start: step.start,
      });
      setOpenStep(null);
    }
    setSelection(null);
  };

  const restore = (id: string) => {
    setCuts((current) => current.filter((item) => item.id !== id));
  };

  /** Steps back one level: the pen, then More, then a mode, then the
   * selection. */
  const back = () => {
    if (tool !== "select") setTool("select");
    else if (isMoreOpen) setIsMoreOpen(false);
    else if (mode) setMode(null);
    else if (openStep !== null) setOpenStep(null);
    else setSelection(null);
  };

  return {
    back,
    backdrop,
    beats,
    cuts,
    effects,
    isMoreOpen,
    isPlaying,
    isVoiced,
    kind,
    lookId,
    map,
    marks,
    mode,
    openStep,
    pickTool,
    place,
    playhead,
    remove,
    restore,
    retimeMark,
    scenes,
    seek,
    select,
    selection,
    setBackdrop,
    setEffects,
    setIsMoreOpen,
    setLookId,
    setMode: (next: Mode) => {
      setMode(next);
      setSelection(null);
      setTool("select");
    },
    setOpenStep,
    setScene: (index: number, layout: Layout) => {
      setScenes((current) =>
        current.map((item, at) => (at === index ? layout : item)),
      );
    },
    setSqueeze,
    setView,
    squeeze,
    togglePlay,
    tool,
    updateMark,
    view,
  };
}

export type Editor = ReturnType<typeof useEditor>;
