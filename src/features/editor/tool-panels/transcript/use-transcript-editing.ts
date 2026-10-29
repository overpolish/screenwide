// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { ToolPanelTranscript } from "../tool-panel-transcript";

import {
  heardSpanOf,
  isHeardInOrder,
  transcriptInPlaybackOrder,
} from "./transcript-playback-order";
import {
  transcriptSceneChoice,
  transcriptSceneMenu,
  transcriptSceneTarget,
} from "./transcript-scenes";
import {
  normalizeText,
  spanOf,
  transcriptUnits,
  WordRange,
  WordSelection,
} from "./transcript-units";
import { TranscriptWord, wordAt } from "./transcript-words";
import { usePendingCorrections } from "./use-pending-corrections";
import { useTranscriptCleanup } from "./use-transcript-cleanup";
import { useTranscriptEditorActions } from "./use-transcript-editor-actions";
import { useTranscriptHighlight } from "./use-transcript-highlight";
import { useTranscriptMenu } from "./use-transcript-menu";
import { useTranscriptMove } from "./use-transcript-move";
import { useTranscriptSearch } from "./use-transcript-search";

import type { TranscriptPanelViewProps } from "./transcript-panel-view";
import type { TranscriptTextProps } from "./transcript-text";

/** What editing the transcript reads of it and asks of the editor. */
export type TranscriptEditingProps = Pick<
  TranscriptPanelViewProps,
  | "isLocked"
  | "language"
  | "onCorrect"
  | "onCut"
  | "onHighlight"
  | "onMove"
  | "onNotProfanity"
  | "onRedo"
  | "onRestore"
  | "onScene"
  | "onSeek"
  | "onTogglePlayback"
  | "onUndo"
  | "plannedSilences"
  | "playheadMs"
  | "sceneOffer"
> & {
  transcript: Pick<
    ToolPanelTranscript,
    | "allowedWords"
    | "corrections"
    | "durationMs"
    | "played"
    | "removed"
    | "scenes"
  >;
  words: readonly TranscriptWord[];
};

/**
 * The transcript as words you choose, cut, correct and move, wherever they
 * are laid out: the words in the order the timeline plays them, what is
 * chosen and being edited, the menu a right click opens, the clean-up
 * tools, and the search for `query`. What it returns for the words is
 * `TranscriptText`'s part (`textProps`); the rest is for the search and the
 * clean-up controls around them.
 */
export function useTranscriptEditing(
  {
    isLocked,
    onCorrect,
    onCut,
    onRestore,
    onScene,
    sceneOffer,
    transcript,
    words,
    ...props
  }: TranscriptEditingProps,
  query: string,
) {
  const [selection, setSelection] = useState<WordRange | null>(null);
  const [editing, setEditing] = useState<WordSelection | null>(null);
  const pending = usePendingCorrections(words, transcript.corrections);
  const heard = useMemo(
    () =>
      transcriptUnits(words, pending.corrections, {
        durationMs: transcript.durationMs,
        removed: transcript.removed,
      }),
    [pending.corrections, transcript.durationMs, transcript.removed, words],
  );
  // The words read in the order the timeline plays them.
  const order = useMemo(
    () => transcriptInPlaybackOrder(heard, transcript.played),
    [heard, transcript.played],
  );
  const { units } = order;
  const cleanup = useTranscriptCleanup({
    allowedWords: transcript.allowedWords,
    durationMs: transcript.durationMs,
    language: props.language,
    onCut,
    onNotProfanity: props.onNotProfanity,
    order,
    plannedSilences: props.plannedSilences,
  });
  const { actions, correct } = useTranscriptEditorActions({
    durationMs: transcript.durationMs,
    onCorrect,
    onCut,
    onRestore,
    order,
    pendingAdd: pending.add,
    setEditing,
  });
  // What a scene picked for the words acts on is worked out again at the
  // pick, so it lands on the timeline as it stands then.
  const sceneTarget = (span: WordSelection) =>
    sceneOffer
      ? transcriptSceneTarget({
          offer: sceneOffer,
          played: transcript.played,
          scenes: transcript.scenes,
          span,
          units,
        })
      : null;
  const openMenu = useTranscriptMenu(
    { ...actions, ...cleanup.menuActions },
    (span, optionId) => {
      const target = sceneTarget(span);
      const choice = sceneOffer
        ? transcriptSceneChoice(sceneOffer, optionId)
        : null;
      if (target && choice) onScene(target, choice);
    },
  );
  const chosen = editing ?? (selection ? spanOf(selection) : null);
  // Only words read as they were heard are one stretch to show.
  useTranscriptHighlight(
    heard,
    chosen && isHeardInOrder(order, chosen) ? heardSpanOf(order, chosen) : null,
    props.onHighlight,
  );
  const search = useTranscriptSearch(units, query, (match) => {
    setSelection({ anchor: match.first, focus: match.last });
    props.onSeek(units[match.first].startMs);
  });
  const moves = useTranscriptMove({
    onMove: props.onMove,
    onRedo: props.onRedo,
    onUndo: props.onUndo,
    order,
    selection,
    setSelection,
  });

  const textProps: TranscriptTextProps = {
    editing,
    isDisabled: isLocked,
    marks: cleanup.marks,
    matches: search.matches,
    onCut: actions.delete,
    onEditEnd: (span, text) => {
      setEditing(null);
      setSelection({ anchor: span.first, focus: span.first });
      if (text === null) return;
      const typed = normalizeText(text);
      const shown = units
        .slice(span.first, span.last + 1)
        .map((unit) => unit.text)
        .join(" ");
      // Nothing left is the words deleted: cut as they were heard.
      if (typed === "") {
        actions.spelling(span);
        actions.delete(span);
      } else if (typed !== normalizeText(shown)) correct(span, typed);
    },
    onEditStart: actions.edit,
    onMenu: (span, point) => {
      const target = sceneTarget(span);
      void openMenu({
        marked: cleanup.menuMark(span),
        point,
        scenes:
          target && sceneOffer ? transcriptSceneMenu(sceneOffer, target) : null,
        span,
        units,
      });
    },
    onMove: moves.move,
    onPaint: (span, cut) => {
      (cut ? actions.delete : actions.restore)(span);
    },
    onRedo: moves.redo,
    onRestore: actions.restore,
    onSeek: props.onSeek,
    onSelectionChange: setSelection,
    onTogglePlayback: props.onTogglePlayback,
    onUndo: moves.undo,
    playingIndex: order.shownIndex[wordAt(heard, props.playheadMs)] ?? -1,
    preview: cleanup.preview,
    selection,
    units,
  };

  return { cleanup, order, search, textProps, units };
}
