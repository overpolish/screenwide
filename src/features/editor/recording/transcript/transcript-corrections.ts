// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A correction and the text it replaces, in whatever unit its caller uses:
 * milliseconds in the Transcript panel, shares of the recording in the edit.
 */
export type TranscriptCorrectionSpan = {
  end: number;
  start: number;
  text: string;
};

/** A word as the transcriber heard it, in the same unit as its corrections. */
export type TranscriptHeardSpan = {
  end: number;
  start: number;
  text: string;
};

/** Text as a reader compares it: runs of spaces as one, none at the ends. */
export const normalizeTranscriptText = (text: string) =>
  text.replace(/\s+/gu, " ").trim();

/**
 * `corrections` once `change` is made: it replaces every correction it
 * overlaps, and where its text is what the words it covers were heard as,
 * it leaves them uncorrected instead. The editor and the Transcript panel
 * both apply this rule, each in its own unit.
 */
export function correctionsWith(
  heard: readonly TranscriptHeardSpan[],
  corrections: readonly TranscriptCorrectionSpan[],
  change: TranscriptCorrectionSpan,
): TranscriptCorrectionSpan[] {
  const kept = corrections.filter(
    (other) => !(other.start < change.end && change.start < other.end),
  );
  const heardText = heard
    .filter((word) => {
      const middle = (word.start + word.end) / 2;
      return change.start <= middle && middle < change.end;
    })
    .map((word) => word.text)
    .join(" ");
  if (
    normalizeTranscriptText(change.text) === normalizeTranscriptText(heardText)
  )
    return kept;
  return [...kept, change].sort((a, b) => a.start - b.start);
}
