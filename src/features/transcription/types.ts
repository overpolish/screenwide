// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** What a model is for. Each use has its own, so nobody has to choose one:
 * transcripts and voice notes take the transcription model, dictation the
 * model for its language and, in English, the model that applies the
 * speaker's corrections, Studio sound its restoration model, Studio ducking
 * its music separation model, Remove background its video matting model,
 * and the camera's objects their tracking model. */
export type TranscriptionPurpose =
  | "cameraBackground"
  | "cameraObjects"
  | "dictation"
  | "dictationCleanup"
  | "studioDucking"
  | "studioSound"
  | "transcripts";

/** A speech model, and how far it is on this computer. */
export type TranscriptionModel = {
  /** What downloading it gets you, in a few words: why it is worth having,
   * never which model it is. */
  description: string;
  id: string;
  /** What it is for, as Settings names it: "Transcripts". */
  name: string;
  /** 0 to 1 while downloading; null otherwise. */
  progress: number | null;
  purpose: TranscriptionPurpose;
  sizeBytes: number;
  status: "available" | "downloaded" | "downloading";
  /** Dictation's model only: whether its words appear as they are spoken,
   * or each time the speaker pauses. */
  writes: "asSpoken" | "atPauses" | null;
};

/** What a spoken command does. */
export type DictationCommandKind =
  | "addWord"
  | "newLine"
  | "newParagraph"
  | "punctuation"
  | "removeWord"
  | "scratchThat";

/** A spoken command, as said in the language dictation is in, with the mark
 * a punctuation command types. */
export type DictationCommand = {
  does: DictationCommandKind;
  said: string;
  mark?: string | null;
};

export type TranscriptionState = {
  /** The spoken commands of the language dictation is in, none in a
   * language without them. */
  dictationCommands: DictationCommand[];
  /** `system`, `auto`, or a language code such as `en`. */
  language: string;
  /** The models something in the app uses, one per purpose. Dictation's is
   * the one for the language, which may be the Transcripts model. */
  models: TranscriptionModel[];
  /** The computer's own language, as a code, which `system` stands for. */
  systemLanguage: string;
  /** The word list: names and terms dictation and transcripts write as
   * spelled here, in order. */
  words: string[];
};
