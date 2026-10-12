// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TranscriptRange } from "../tool-panel-transcript";
import type { TranscriptWord } from "./transcript-words";

/** A silence at least this long becomes a pause unit in the transcript:
 * about the shortest pause a listener hears as one. Gaps between words only
 * exist where the speech detector heard the voice stop, so this never marks
 * a pause inside continuous speech; it stays just above a breath at a
 * comma. */
const PAUSE_MIN_MS = 300;

/**
 * The silences at least `minMs` long, as pauses: each gap between
 * consecutive words, and, where the recording's length is known, the
 * silence before the first word and after the last, so a slow start or a
 * long tail can be cut from the transcript too. `words` is sorted by start
 * and its words never overlap.
 */
export function transcriptPauses(
  words: readonly TranscriptWord[],
  {
    durationMs,
    minMs = PAUSE_MIN_MS,
  }: { durationMs?: number; minMs?: number } = {},
): TranscriptRange[] {
  const pauses: TranscriptRange[] = [];
  const add = (startMs: number, endMs: number) => {
    if (endMs - startMs >= minMs) pauses.push({ endMs, startMs });
  };
  if (durationMs !== undefined) add(0, words[0]?.startMs ?? durationMs);
  for (let index = 1; index < words.length; index++)
    add(words[index - 1].endMs, words[index].startMs);
  const last = words.length > 0 ? words[words.length - 1] : undefined;
  if (durationMs !== undefined && last) add(last.endMs, durationMs);
  return pauses;
}

/** The indices of the pause units, still kept, that a cut in `cuts` reaches:
 * the pauses Remove silences would act on. */
export function pausesCutBy(
  units: readonly (TranscriptRange & {
    kind: "pause" | "word";
    removed: boolean;
  })[],
  cuts: readonly TranscriptRange[],
): ReadonlySet<number> {
  const reached = new Set<number>();
  units.forEach((unit, index) => {
    if (unit.kind !== "pause" || unit.removed) return;
    if (
      cuts.some((cut) => cut.startMs < unit.endMs && cut.endMs > unit.startMs)
    )
      reached.add(index);
  });
  return reached;
}
