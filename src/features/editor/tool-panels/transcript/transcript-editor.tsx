// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { SilencesRow } from "../silences-row";
import { ToolPanelTranscript } from "../tool-panel-transcript";

import { TranscriptHint } from "./transcript-hint";
import { TranscriptMarksRow } from "./transcript-marks-row";
import { TranscriptActionsRow } from "./transcript-options-rows";
import {
  heardSpanOf,
  isHeardInOrder,
  transcriptInPlaybackOrder,
} from "./transcript-playback-order";
import { TranscriptReplaceRow } from "./transcript-replace-row";
import {
  transcriptSceneChoice,
  transcriptSceneMenu,
  transcriptSceneTarget,
} from "./transcript-scenes";
import { TranscriptText } from "./transcript-text";
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

/** The search, the words, the help line, the clean-up rows, and the way
 * back from every cut under them. */
export function TranscriptEditor({
  isLocked,
  onCorrect,
  onCut,
  onRestore,
  onRestoreAll,
  onScene,
  sceneOffer,
  transcript,
  words,
  ...props
}: TranscriptPanelViewProps & {
  transcript: ToolPanelTranscript;
  words: readonly TranscriptWord[];
}) {
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
  const wordsRef = useRef<HTMLDivElement>(null);
  const search = useTranscriptSearch(units, (match) => {
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

  return (
    <div className="flex flex-col gap-section" ref={wordsRef}>
      <TranscriptReplaceRow
        isDisabled={isLocked}
        matches={search.matches.filter((match) => isHeardInOrder(order, match))}
        onCorrectMany={props.onCorrectMany}
        onLeave={() => {
          wordsRef.current
            ?.querySelector<HTMLElement>('[role="listbox"]')
            ?.focus();
        }}
        onQueryChange={search.setQuery}
        onStep={search.step}
        query={search.query}
        units={units}
      />
      <div className="flex flex-col gap-control">
        <TranscriptText
          editing={editing}
          isDisabled={isLocked}
          marks={cleanup.marks}
          matches={search.matches}
          onCut={actions.delete}
          onEditEnd={(span, text) => {
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
          }}
          onEditStart={actions.edit}
          onMenu={(span, point) => {
            const target = sceneTarget(span);
            void openMenu({
              marked: cleanup.menuMark(span),
              point,
              scenes:
                target && sceneOffer
                  ? transcriptSceneMenu(sceneOffer, target)
                  : null,
              span,
              units,
            });
          }}
          onMove={moves.move}
          onPaint={(span, cut) => {
            (cut ? actions.delete : actions.restore)(span);
          }}
          onRedo={moves.redo}
          onRestore={actions.restore}
          onSeek={props.onSeek}
          onSelectionChange={setSelection}
          onTogglePlayback={props.onTogglePlayback}
          onUndo={moves.undo}
          playingIndex={order.shownIndex[wordAt(heard, props.playheadMs)] ?? -1}
          preview={cleanup.preview}
          selection={selection}
          units={units}
        />
        <TranscriptHint />
      </div>
      {transcript.silences ? (
        <SilencesRow
          isLocked={isLocked}
          onPreview={cleanup.previewers.silences}
          onRemove={props.onRemoveSilences}
          silences={transcript.silences}
        />
      ) : null}
      <TranscriptMarksRow
        count={cleanup.fillers.length}
        isLocked={isLocked}
        onPreview={cleanup.previewers.fillers}
        onRemove={cleanup.removeFillers}
        title={t("editor-panels-transcript-fillers")}
      />
      <TranscriptMarksRow
        count={cleanup.profanity.length}
        isLocked={isLocked}
        onPreview={cleanup.previewers.profanity}
        onRemove={cleanup.removeProfanity}
        title={t("editor-panels-transcript-profanity")}
      />
      <TranscriptActionsRow
        canRestore={transcript.canRestore}
        isLocked={isLocked}
        onRestoreAll={onRestoreAll}
        onTranscribeAgain={props.onTranscribe}
      />
    </div>
  );
}
