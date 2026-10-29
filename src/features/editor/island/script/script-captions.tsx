// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandKeyHint } from "../../../../primitives/island/island-key-hint";
import { CaptionTextWord } from "../../tool-panels/captions/caption-text";
import { CaptionTextView } from "../../tool-panels/captions/caption-text-view";
import { useCaptionWordEditing } from "../../tool-panels/captions/use-caption-word-editing";

import type { ScriptCaptionsProps } from "./script-island-props";

/** The script as the captions break it: every word the timeline plays, a
 * mark wherever one caption ends and the next begins, in another colour
 * where that was placed by hand, and the caption being said tinted and
 * kept in view. It scrolls within the island's height rather than running
 * the island down past the picture. Under it, how to edit it, and taking
 * back every edit made in it: what was placed by hand, and spelling, which
 * goes back to what was heard in the transcript too. */
export function ScriptCaptions({
  captions,
  durationMs,
  marks,
  onCaptionsChange,
  onCorrect,
  onSeek,
  playheadMs,
  text,
}: ScriptCaptionsProps) {
  const correction = (word: CaptionTextWord, corrected: string) => ({
    endMs: word.endMs,
    startMs: word.startMs,
    text: corrected,
  });
  const { hasPlaced, reset, ...wordEditing } = useCaptionWordEditing({
    captions,
    durationMs,
    isDisabled: false,
    marks,
    onChange: onCaptionsChange,
    onCorrect: (word, corrected) => {
      onCorrect([correction(word, corrected)]);
    },
    text,
  });
  const corrected = text.words.filter((word) => word.corrected);
  return (
    <div className="flex w-80 flex-col gap-separate">
      <CaptionTextView
        {...wordEditing}
        className="px-inset-sm py-0"
        isDisabled={false}
        marks={marks}
        onSeek={onSeek}
        playheadMs={playheadMs}
        rootClassName="max-h-[26rem] rounded-none bg-transparent"
        text={text}
      />
      <IslandKeyHint
        binding="CommandOrControl"
        message={(key) => t("editor-island-script-captions-hint", { key })}
      />
      <div className="flex justify-end">
        <Button
          isDisabled={!hasPlaced && corrected.length === 0}
          onPress={() => {
            if (hasPlaced) reset();
            if (corrected.length > 0)
              onCorrect(corrected.map((word) => correction(word, word.heard)));
          }}
        >
          {t("editor-island-script-reset")}
        </Button>
      </div>
    </div>
  );
}
