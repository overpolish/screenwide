// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { SettingsGroup } from "../settings-group";
import { PreviewArea } from "../stage/preview-stage";

import { dictationCommands } from "./dictation-rows";
import { WordsSettings } from "./words-settings";

import type { TranscriptionControls } from "./use-transcription";
import type { DictationSubject } from "../stage/dictation-script";
import type { PreviewFollow } from "../stage/use-preview-subject";

// Dictation among the tools: the word list it shares with recording
// transcripts and voice notes, and the commands it takes, which are only
// read here, since they are said rather than set. Its model is in Models
// and its shortcut in Shortcuts, with the others. Each row drives the
// picture: a word added, or a command carried out.
export function DictationSettingsPanel({
  controls,
  follow,
}: {
  controls: TranscriptionControls;
  follow: (subject: DictationSubject) => PreviewFollow;
}) {
  return (
    <>
      <PreviewArea follow={follow("addWord")}>
        <WordsSettings controls={controls} />
      </PreviewArea>
      <SettingsGroup
        description={t("settings-dictation-commands-description")}
        title={t("settings-dictation-commands")}
      >
        {dictationCommands().map(({ description, subject, title }) => (
          <PreviewArea follow={follow(subject)} key={subject}>
            <ControlRow description={description} title={title}>
              {() => null}
            </ControlRow>
          </PreviewArea>
        ))}
      </SettingsGroup>
    </>
  );
}
