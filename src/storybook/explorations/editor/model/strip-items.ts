// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Beat, beatAt } from "../recording";

import { type Editor, type Selection } from "./use-editor";

// The strip's own row, beats and the pauses between them, in order: what
// the arrow keys step through, and where the lens opens for a selection.

type Item = {
  end: number;
  id: string;
  kind: "beat" | "pause";
  /** Where choosing it takes the playhead, in recorded seconds. */
  seconds: number;
  start: number;
};

/** The lines drawn in the voice row: those that are not a beat of their
 * own on the strip. */
export const captionLines = (editor: Editor) =>
  editor.recording.speech.filter(
    (line) =>
      !editor.beats.some(
        (beat) => beat.kind === "speech" && beat.start === line.start,
      ),
  );

/** Whether what a beat draws over the picture is hidden: a shortcut, with
 * the shortcuts turned off. */
export const isOverlayHidden = (editor: Editor, beat: Beat) =>
  !editor.effects.keys && beat.kind === "action" && beat.action.kind === "keys";

function itemsOf(editor: Editor): Item[] {
  const isCut = ({ end, start }: { end: number; start: number }) =>
    editor.cuts.some((cut) => cut.start <= start && cut.end >= end);
  const beats = editor.beats.filter((beat) => !isCut(beat));
  return [
    ...beats.map((beat): Item => ({
      end: beat.end,
      id: beat.id,
      kind: "beat",
      seconds: beatAt(beat),
      start: beat.start,
    })),
    ...editor.map.pauses.map((pause): Item => ({
      ...pause,
      kind: "pause",
      seconds: pause.start,
    })),
  ].sort((first, second) => first.start - second.start);
}

/** Chooses the beat or pause beside the chosen one, or with nothing
 * chosen, the one under the playhead. Annotations and scenes have rails of
 * their own, so the keys leave them be. */
export function chooseNeighbour(editor: Editor, direction: -1 | 1) {
  const { playhead, selection } = editor;
  if (selection && selection.kind !== "beat" && selection.kind !== "pause")
    return;
  const items = itemsOf(editor);
  const index = selection
    ? items.findIndex(
        (item) => item.kind === selection.kind && item.id === selection.id,
      )
    : -1;
  const next =
    index < 0
      ? items.find((item) => playhead >= item.start && playhead < item.end)
      : items[index + direction];
  if (!next) return;
  editor.select({ id: next.id, kind: next.kind });
  editor.seek(next.seconds);
}

/** The stretch the island stands over, in recorded seconds: what is
 * chosen, or what a mode was opened from on the strip, such as a caption.
 * A new choice also brings it into view. */
export function focusOf(
  editor: Editor,
  selection: Selection,
): { end: number; start: number } | null {
  switch (selection?.kind) {
    case "beat":
      return editor.beats.find(({ id }) => id === selection.id) ?? null;
    case "mark":
      return editor.marks.find(({ id }) => id === selection.id) ?? null;
    case "pause":
      return editor.map.pauses.find(({ id }) => id === selection.id) ?? null;
    case "range":
      return selection;
    case "scene":
      return editor.scenes.find(({ id }) => id === selection.id) ?? null;
    case undefined:
      return editor.mode ? (editor.modeFrom ?? null) : null;
  }
}
