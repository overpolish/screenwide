// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** What a model transcribes. Each use has its own, so nobody has to choose
 * one: moments take the quick model, recordings the accurate one. */
export type TranscriptionPurpose = "moments" | "recordings";

/** A speech-to-text model, and how far it is on this computer. */
export type TranscriptionModel = {
  /** What downloading it gets you, in a few words: why it is worth having,
   * never which model it is. */
  description: string;
  id: string;
  /** What it is for, as Settings names it: "Moments". */
  name: string;
  /** 0 to 1 while downloading; null otherwise. */
  progress: number | null;
  purpose: TranscriptionPurpose;
  sizeBytes: number;
  status: "available" | "downloaded" | "downloading";
};

export type TranscriptionState = {
  /** `system`, `auto`, or a language code such as `en`. */
  language: string;
  /** The models something in the app uses, one per purpose. */
  models: TranscriptionModel[];
  /** The computer's own language, as a code, which `system` stands for. */
  systemLanguage: string;
};
