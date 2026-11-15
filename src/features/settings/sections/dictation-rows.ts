// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { DictationCommandKind } from "../../transcription/types";
import type { DictationSubject } from "../stage/dictation-script";

/** The kinds of spoken command, in the order Tools' Dictation lists them,
 * each with the picture that shows it carried out. */
const KINDS: { does: DictationCommandKind; subject: DictationSubject }[] = [
  { does: "scratchThat", subject: "scratch" },
  { does: "newLine", subject: "newLine" },
  { does: "newParagraph", subject: "newParagraph" },
  { does: "punctuation", subject: "punctuation" },
  { does: "addWord", subject: "addWord" },
  { does: "removeWord", subject: "removeWord" },
];

const title = (does: DictationCommandKind) => {
  switch (does) {
    case "scratchThat":
      return t("settings-dictation-command-scratch");
    case "newLine":
      return t("settings-dictation-command-new-line");
    case "newParagraph":
      return t("settings-dictation-command-new-paragraph");
    case "punctuation":
      return t("settings-dictation-command-punctuation");
    case "addWord":
      return t("settings-dictation-command-add-word");
    case "removeWord":
      return t("settings-dictation-command-remove-word");
  }
};

/** Each kind of spoken command, named for what it does in the app's
 * language, as Tools' Dictation lists them and search finds them. What to
 * say comes from the language dictation is in. */
export const dictationCommands = () =>
  KINDS.map((kind) => ({ ...kind, title: title(kind.does) }));
