// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Beat, RECORDING_S, SPEECH } from "../editor-data";

// How the edited recording's time relates to the recorded time. The
// recording is cut into pieces wherever a beat, a spoken line or a cut
// starts or ends. A cut piece plays for no time. A squeeze takes time from
// still pieces only: nothing being done and, when there is a voice, nothing
// being said. The longest give first, levelled down together, and none goes
// below a breath.

export type Cut = { end: number; id: string; start: number };

export type Piece = {
  end: number;
  isCut: boolean;
  isSpeech: boolean;
  start: number;
  beat?: Beat;
};

export type TimeMap = {
  /** The edited recording's length, in seconds. */
  duration: number;
  /** The most a squeeze can take, in seconds. */
  maxSqueeze: number;
  pieces: Piece[];
  /** The recorded moment at a moment of the edited recording. */
  toOriginal: (time: number) => number;
  /** Where a recorded moment falls in the edited recording. */
  toTime: (seconds: number) => number;
};

const BREATH_S = 0.3;

const contains = (span: { end: number; start: number }, at: number) =>
  at >= span.start && at < span.end;

function piecesOf(beats: Beat[], isVoiced: boolean, cuts: Cut[]): Piece[] {
  const spans = [...beats, ...(isVoiced ? SPEECH : []), ...cuts];
  const edges = [
    ...new Set([
      0,
      RECORDING_S,
      ...spans.flatMap(({ end, start }) => [start, end]),
    ]),
  ]
    .filter((edge) => edge >= 0 && edge <= RECORDING_S)
    .sort((first, second) => first - second);
  return edges.slice(1).map((end, index) => {
    const start = edges[index];
    const middle = (start + end) / 2;
    return {
      beat: beats.find((beat) => contains(beat, middle)),
      end,
      isCut: cuts.some((cut) => contains(cut, middle)),
      isSpeech: isVoiced && SPEECH.some((line) => contains(line, middle)),
      start,
    };
  });
}

const isStill = (piece: Piece) =>
  !piece.beat && !piece.isSpeech && !piece.isCut;

export function timeMap({
  beats,
  cuts,
  isVoiced,
  squeeze,
}: {
  beats: Beat[];
  cuts: Cut[];
  isVoiced: boolean;
  squeeze: number;
}): TimeMap {
  const pieces = piecesOf(beats, isVoiced, cuts);
  const length = (piece: Piece) => piece.end - piece.start;
  const still = pieces.filter(isStill);
  const maxSqueeze = still.reduce(
    (sum, piece) => sum + Math.max(0, length(piece) - BREATH_S),
    0,
  );
  const amount = Math.min(Math.max(squeeze, 0), maxSqueeze);
  const keptAt = (level: number, piece: Piece) =>
    Math.min(length(piece), Math.max(level, BREATH_S));
  let low = 0;
  let high = Math.max(0, ...still.map(length));
  for (let index = 0; index < 32 && amount > 0; index += 1) {
    const level = (low + high) / 2;
    const taken = still.reduce(
      (sum, piece) => sum + length(piece) - keptAt(level, piece),
      0,
    );
    if (taken > amount) low = level;
    else high = level;
  }
  const kept = pieces.map((piece) =>
    piece.isCut
      ? 0
      : isStill(piece) && amount > 0
        ? keptAt(high, piece)
        : length(piece),
  );
  const starts = kept.reduce<number[]>(
    (all, value, index) => [...all, all[index] + value],
    [0],
  );
  const duration = starts[pieces.length];

  const toTime = (seconds: number) => {
    const index = pieces.findIndex((piece) => seconds < piece.end);
    if (index < 0) return duration;
    const piece = pieces[index];
    const into = Math.max(0, seconds - piece.start) / length(piece);
    return starts[index] + into * kept[index];
  };
  const toOriginal = (time: number) => {
    const index = pieces.findIndex(
      (_, at) => kept[at] > 0 && time < starts[at] + kept[at],
    );
    if (index < 0) return RECORDING_S;
    const piece = pieces[index];
    const into = Math.max(0, time - starts[index]) / kept[index];
    return piece.start + into * length(piece);
  };
  return { duration, maxSqueeze, pieces, toOriginal, toTime };
}
