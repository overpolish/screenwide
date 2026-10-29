// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioLinesOff } from "lucide-react";
import { useEffect, useMemo, useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { IslandKeyHint } from "../../../../primitives/island/island-key-hint";
import { Text } from "../../../../primitives/text/text";
import { pausesCutBy } from "../../tool-panels/transcript/transcript-pauses";
import { isHeardInOrder } from "../../tool-panels/transcript/transcript-playback-order";
import { replacementCorrections } from "../../tool-panels/transcript/transcript-replace";
import { TranscriptText } from "../../tool-panels/transcript/transcript-text";
import { useTranscriptEditing } from "../../tool-panels/transcript/use-transcript-editing";
import { FillerGlyph, ProfanityGlyph } from "../captions/caption-glyphs";
import { IslandWorkLine } from "../island-work-line";

import { clearScriptSearch, useScriptSearch } from "./script-search";
import { ScriptRemoveChip, ScriptReplaceLine } from "./script-transcript-parts";

import type { ScriptTranscriptProps } from "./script-island-props";

/**
 * The script on its own, to cut: Replace under the row while it is open,
 * then the words as the timeline plays them, cut ones struck through and
 * pauses as chips, chosen, removed, moved and put right in place. Under
 * them, how to edit them; the clean-up that removes the pauses, fillers
 * and profanity at once, each showing what it would take while hovered;
 * and transcribing again or bringing back every cut.
 */
export function ScriptTranscript({
  onCorrect,
  script,
  ...props
}: ScriptTranscriptProps) {
  const { isLocked, onRemoveSilences, plannedSilences } = props;
  const query = useScriptSearch((state) => state.query);
  const isReplacing = useScriptSearch((state) => state.isReplacing);
  const { cleanup, order, search, textProps, units } = useTranscriptEditing(
    {
      ...props,
      onCorrect: (correction) => {
        onCorrect([correction]);
      },
      transcript: script,
    },
    query,
  );
  const rootRef = useRef<HTMLDivElement>(null);
  const stepRef = useRef(search.step);
  stepRef.current = search.step;

  // The row's search goes to a match, and gives the keys back, through
  // the words; it is put away with them.
  useEffect(() => {
    useScriptSearch.setState({
      words: {
        focus: () => {
          rootRef.current
            ?.querySelector<HTMLElement>('[role="listbox"]')
            ?.focus();
        },
        step: (by) => {
          stepRef.current(by);
        },
      },
    });
    return clearScriptSearch;
  }, []);
  const found = search.matches.length;
  useEffect(() => {
    useScriptSearch.setState({ found });
  }, [found]);

  const pauses = useMemo(
    () => pausesCutBy(order.heard, plannedSilences).size,
    [order.heard, plannedSilences],
  );
  const { silences } = script;
  const isFinding = silences?.status === "finding";

  // Every part is in one flow, so Replace coming in or going moves what is
  // under it along with the island's black rather than jumping.
  return (
    <div className="w-80" ref={rootRef}>
      <IslandFlow className="flex flex-col gap-separate">
        {isReplacing ? (
          <ScriptReplaceLine
            found={found}
            isDisabled={isLocked}
            key="replace"
            onReplaceAll={(replacement) => {
              const corrections = replacementCorrections(
                units,
                search.matches.filter((match) => isHeardInOrder(order, match)),
                replacement,
              );
              if (corrections.length > 0) onCorrect(corrections);
            }}
          />
        ) : null}
        <TranscriptText
          key="words"
          {...textProps}
          className="px-inset-sm py-0"
          rootClassName="max-h-[26rem] rounded-none bg-transparent"
        />
        <IslandKeyHint
          binding="CommandOrControl"
          key="hint"
          message={(key) => t("editor-island-script-transcript-hint", { key })}
        />
        {isFinding ? (
          <IslandWorkLine
            key="finding"
            label={t("editor-island-script-finding-pauses")}
            value={silences.progress ?? 0}
          />
        ) : silences?.status === "none-found" ? (
          <Text
            as="p"
            className="px-inset-sm text-fg-secondary"
            key="no-pauses"
            variant="caption"
          >
            {t("editor-island-script-no-pauses")}
          </Text>
        ) : null}
        {/* Each chip shows only while it has something to take; the line
            goes with the last. */}
        {pauses + cleanup.fillers.length + cleanup.profanity.length > 0 ? (
          <IslandFlow className="flex items-center gap-inline" key="clean-up">
            {silences && pauses > 0 ? (
              <ScriptRemoveChip
                count={pauses}
                glyph={() => <AudioLinesOff aria-hidden />}
                isDisabled={isLocked || isFinding}
                key="pauses"
                label={t("editor-island-script-remove-pauses", {
                  count: pauses,
                })}
                onPreview={cleanup.previewers.silences}
                onRemove={onRemoveSilences}
              />
            ) : null}
            {cleanup.fillers.length > 0 ? (
              <ScriptRemoveChip
                count={cleanup.fillers.length}
                glyph={(isPreviewing) => <FillerGlyph isOn={isPreviewing} />}
                isDisabled={isLocked}
                key="fillers"
                label={t("editor-island-script-remove-fillers", {
                  count: cleanup.fillers.length,
                })}
                onPreview={cleanup.previewers.fillers}
                onRemove={cleanup.removeFillers}
              />
            ) : null}
            {cleanup.profanity.length > 0 ? (
              <ScriptRemoveChip
                count={cleanup.profanity.length}
                glyph={(isPreviewing) => <ProfanityGlyph isOn={isPreviewing} />}
                isDisabled={isLocked}
                key="profanity"
                label={t("editor-island-script-remove-profanity", {
                  count: cleanup.profanity.length,
                })}
                onPreview={cleanup.previewers.profanity}
                onRemove={cleanup.removeProfanity}
              />
            ) : null}
          </IslandFlow>
        ) : null}
        <div className="flex justify-end gap-separate" key="actions">
          <Button isDisabled={isLocked} onPress={props.onTranscribe}>
            {t("editor-island-script-transcribe-again")}
          </Button>
          <Button
            isDisabled={isLocked || !script.canRestore}
            onPress={props.onRestoreAll}
          >
            {t("editor-island-script-restore-all")}
          </Button>
        </div>
      </IslandFlow>
    </div>
  );
}
