// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { SilencesRow } from "../silences-row";
import { ToolPanelTranscript } from "../tool-panel-transcript";

import { TranscriptHint } from "./transcript-hint";
import { TranscriptMarksRow } from "./transcript-marks-row";
import { TranscriptActionsRow } from "./transcript-options-rows";
import { isHeardInOrder } from "./transcript-playback-order";
import { TranscriptReplaceRow } from "./transcript-replace-row";
import { TranscriptText } from "./transcript-text";
import { TranscriptWord } from "./transcript-words";
import { useTranscriptEditing } from "./use-transcript-editing";

import type { TranscriptPanelViewProps } from "./transcript-panel-view";

/** The search, the words, the help line, the clean-up rows, and the way
 * back from every cut under them. */
export function TranscriptEditor(
  props: TranscriptPanelViewProps & {
    transcript: ToolPanelTranscript;
    words: readonly TranscriptWord[];
  },
) {
  const { isLocked, transcript } = props;
  const [query, setQuery] = useState("");
  const { cleanup, order, search, textProps, units } = useTranscriptEditing(
    props,
    query,
  );
  const wordsRef = useRef<HTMLDivElement>(null);

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
        onQueryChange={setQuery}
        onStep={search.step}
        query={query}
        units={units}
      />
      <div className="flex flex-col gap-control">
        <TranscriptText {...textProps} />
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
        onRestoreAll={props.onRestoreAll}
        onTranscribeAgain={props.onTranscribe}
      />
    </div>
  );
}
