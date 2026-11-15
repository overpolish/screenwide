// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useListFormatter } from "react-aria";

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { SettingsGroup } from "../settings-group";
import { PreviewArea } from "../stage/preview-stage";

import { dictationCommands } from "./dictation-rows";
import { WordsSettings } from "./words-settings";

import type { TranscriptionControls } from "./use-transcription";
import type { DictationCommand } from "../../transcription/types";
import type { DictationSubject } from "../stage/dictation-script";
import type { PreviewFollow } from "../stage/use-preview-subject";

// Dictation among the tools: the word list it shares with recording
// transcripts and voice notes, and the commands it takes, which are only
// read here, since they are said rather than set. Each command is named
// for what it does in the app's language, with what to say in the language
// dictation is in, which may differ; a language without commands lists
// none. Its model is in Models and its shortcut in Shortcuts, with the
// others. Each row drives the picture: a word added, or a command carried
// out.
export function DictationSettingsPanel({
  controls,
  follow,
}: {
  controls: TranscriptionControls;
  follow: (subject: DictationSubject) => PreviewFollow;
}) {
  const alternatives = useListFormatter({ type: "disjunction" });
  const commands: DictationCommand[] = controls.state?.dictationCommands ?? [];
  const quoted = (does: DictationCommand["does"]) =>
    commands
      .filter((command) => command.does === does)
      .map((command) =>
        t("settings-dictation-command-phrase", { phrase: command.said }),
      );
  const rows = dictationCommands().flatMap((row) => {
    const phrases = quoted(row.does);
    if (phrases.length === 0) return [];
    const say = alternatives.format(phrases);
    const description =
      row.does === "addWord" || row.does === "removeWord"
        ? t("settings-dictation-command-say-word", { phrases: say })
        : t("settings-dictation-command-say", { phrases: say });
    return [{ ...row, description }];
  });
  const [addWord] = quoted("addWord");
  return (
    <>
      <PreviewArea follow={follow("addWord")}>
        <WordsSettings
          controls={controls}
          footnote={
            addWord
              ? t("settings-words-by-voice", { phrase: addWord })
              : undefined
          }
        />
      </PreviewArea>
      {rows.length > 0 ? (
        <SettingsGroup
          description={t("settings-dictation-commands-description")}
          title={t("settings-dictation-commands")}
        >
          {rows.map(({ description, subject, title }) => (
            <PreviewArea follow={follow(subject)} key={subject}>
              <ControlRow description={description} title={title}>
                {() => null}
              </ControlRow>
            </PreviewArea>
          ))}
        </SettingsGroup>
      ) : null}
    </>
  );
}
