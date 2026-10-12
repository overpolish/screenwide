// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { SilencesRow } from "../silences-row";
import { ToolPanelTranscript } from "../tool-panel-transcript";

import { TranscriptHint } from "./transcript-hint";
import { TranscriptMarksRow } from "./transcript-marks-row";
import { TranscriptActionsRow } from "./transcript-options-rows";
import { TranscriptReplaceRow } from "./transcript-replace-row";
import { TranscriptText } from "./transcript-text";
import {
  normalizeText,
  spanOf,
  transcriptUnits,
  WordRange,
  WordSelection,
} from "./transcript-units";
import { TranscriptWord } from "./transcript-words";
import { usePendingCorrections } from "./use-pending-corrections";
import { useTranscriptCleanup } from "./use-transcript-cleanup";
import { useTranscriptEditorActions } from "./use-transcript-editor-actions";
import { useTranscriptHighlight } from "./use-transcript-highlight";
import { useTranscriptMenu } from "./use-transcript-menu";
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
  const units = useMemo(
    () =>
      transcriptUnits(words, pending.corrections, {
        durationMs: transcript.durationMs,
        removed: transcript.removed,
      }),
    [pending.corrections, transcript.durationMs, transcript.removed, words],
  );
  const cleanup = useTranscriptCleanup({
    allowedWords: transcript.allowedWords,
    durationMs: transcript.durationMs,
    language: props.language,
    onCut,
    onNotProfanity: props.onNotProfanity,
    plannedSilences: props.plannedSilences,
    units,
  });
  const { actions, correct } = useTranscriptEditorActions({
    durationMs: transcript.durationMs,
    onCorrect,
    onCut,
    onRestore,
    pendingAdd: pending.add,
    setEditing,
    units,
  });
  const openMenu = useTranscriptMenu({ ...actions, ...cleanup.menuActions });
  useTranscriptHighlight(
    units,
    editing ?? (selection ? spanOf(selection) : null),
    props.onHighlight,
  );
  const wordsRef = useRef<HTMLDivElement>(null);
  const search = useTranscriptSearch(units, (match) => {
    setSelection({ anchor: match.first, focus: match.last });
    props.onSeek(units[match.first].startMs);
  });

  return (
    <div className="flex flex-col gap-section" ref={wordsRef}>
      <TranscriptReplaceRow
        isDisabled={isLocked}
        matches={search.matches}
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
          void openMenu({ marked: cleanup.menuMark(span), point, span, units });
        }}
        onPaint={(span, cut) => {
          (cut ? actions.delete : actions.restore)(span);
        }}
        onRedo={props.onRedo}
        onRestore={actions.restore}
        onSeek={props.onSeek}
        onSelectionChange={setSelection}
        onTogglePlayback={props.onTogglePlayback}
        onUndo={props.onUndo}
        playheadMs={props.playheadMs}
        preview={cleanup.preview}
        selection={selection}
        units={units}
      />
      <TranscriptHint />
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
