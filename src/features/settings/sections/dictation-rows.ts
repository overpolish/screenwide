// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { DictationSubject } from "../stage/dictation-script";

/** The spoken commands, as Tools' Dictation lists them and search finds
 * them, each with the picture that shows it carried out. */
export const dictationCommands = (): {
  description: string;
  subject: DictationSubject;
  title: string;
}[] => [
  {
    description: t("settings-dictation-command-scratch-description"),
    subject: "scratch",
    title: t("settings-dictation-command-scratch"),
  },
  {
    description: t("settings-dictation-command-new-line-description"),
    subject: "newLine",
    title: t("settings-dictation-command-new-line"),
  },
  {
    description: t("settings-dictation-command-new-paragraph-description"),
    subject: "newParagraph",
    title: t("settings-dictation-command-new-paragraph"),
  },
  {
    description: t("settings-dictation-command-punctuation-description"),
    subject: "punctuation",
    title: t("settings-dictation-command-punctuation"),
  },
  {
    description: t("settings-dictation-command-add-word-description"),
    subject: "addWord",
    title: t("settings-dictation-command-add-word"),
  },
  {
    description: t("settings-dictation-command-remove-word-description"),
    subject: "removeWord",
    title: t("settings-dictation-command-remove-word"),
  },
];
