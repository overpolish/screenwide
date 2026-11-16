// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type {
  DictationCommand,
  TranscriptionModel,
  TranscriptionState,
} from "../transcription/types";

/** The Dictation models for English, the streaming model and the one that
 * writes each stretch again, at their real download size together. */
const DICTATION_MODEL: TranscriptionModel = {
  description: "Types what you say into any app as you speak.",
  id: "parakeet-unified-en-0.6b-q8_0",
  name: "Dictation",
  progress: null,
  purpose: "dictation",
  sizeBytes: 1_471_720_736,
  status: "available",
  writes: "asSpoken",
};

/** The Dictation model for European languages, at its real download size. */
const EUROPEAN_DICTATION_MODEL: TranscriptionModel = {
  description: "Types what you say into any app each time you pause.",
  id: "parakeet-ultra-q8_0",
  name: "Dictation",
  progress: null,
  purpose: "dictation",
  sizeBytes: 740_363_168,
  status: "available",
  writes: "atPauses",
};

/** The model that applies English dictation's corrections, at its real
 * download size. */
const DICTATION_CLEANUP_MODEL: TranscriptionModel = {
  description:
    "Applies corrections you make as you speak, such as “2, actually 3”.",
  id: "speakoflow-mini-0.8b-q8_0",
  name: "Dictation corrections",
  progress: null,
  purpose: "dictationCleanup",
  sizeBytes: 833_591_776,
  status: "available",
  writes: null,
};

/** The languages the European model dictates, as the app's catalogue has
 * them. */
const EUROPEAN = new Set(
  "bg cs da de es et fi fr hr hu it lt lv mt nl pl pt ro ru sk sl sv uk".split(
    " ",
  ),
);

const ENGLISH_COMMANDS: DictationCommand[] = [
  { does: "scratchThat", said: "scratch that" },
  { does: "newLine", said: "new line" },
  { does: "newLine", said: "next line" },
  { does: "newParagraph", said: "new paragraph" },
  { does: "punctuation", mark: ",", said: "comma" },
  { does: "punctuation", mark: ".", said: "full stop" },
  { does: "punctuation", mark: "?", said: "question mark" },
  { does: "addWord", said: "add word" },
  { does: "removeWord", said: "remove word" },
];

const FRENCH_COMMANDS: DictationCommand[] = [
  { does: "scratchThat", said: "efface ça" },
  { does: "newLine", said: "à la ligne" },
  { does: "newLine", said: "nouvelle ligne" },
  { does: "newParagraph", said: "nouveau paragraphe" },
  { does: "punctuation", mark: ",", said: "virgule" },
  { does: "punctuation", mark: ".", said: "point" },
  { does: "punctuation", mark: "?", said: "point d'interrogation" },
  { does: "addWord", said: "ajouter le mot" },
  { does: "removeWord", said: "supprimer le mot" },
];

/** Dictation's models and commands for `language` as the app chooses them:
 * English streams and has its corrections applied, the European model
 * writes its languages at each pause, and the Transcripts model every other
 * language. */
export const dictationFor = (
  language: string,
  transcripts: TranscriptionModel,
): Pick<TranscriptionState, "dictationCommands"> & {
  models: TranscriptionModel[];
} => {
  const code = language === "system" ? "en" : language;
  if (code === "en")
    return {
      dictationCommands: ENGLISH_COMMANDS,
      models: [DICTATION_MODEL, DICTATION_CLEANUP_MODEL],
    };
  if (EUROPEAN.has(code))
    return {
      dictationCommands: code === "fr" ? FRENCH_COMMANDS : [],
      models: [EUROPEAN_DICTATION_MODEL],
    };
  return {
    dictationCommands: [],
    models: [
      {
        ...transcripts,
        description:
          "Types what you say into any app each time you pause, with the Transcripts model.",
        name: "Dictation",
        purpose: "dictation",
        writes: "atPauses",
      },
    ],
  };
};
