// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Beat, type Recording } from "../recording";

import { CAPS, ownPlay } from "./speed";

// How the edited recording's time relates to the recorded time. The
// recording is cut into pieces wherever a beat, a spoken line, a stretch of
// picture or a cut starts or ends. A cut piece plays for no time. Between
// beats are pauses, each a run of pieces where nothing holds; what they
// have is give. Still pieces, where nothing is said and the picture stands
// still, give most; moving pieces, where the picture moves by itself, give
// a little, and only when the pause is retimed by hand: the squeeze never
// hurries what moves, since a loading bar and a preview playing look the
// same to it. Give only ever changes speed, as the spring drawn for a pause
// says, as fast as the picture allows (`CAPS`); taking time away is a cut,
// and a cut leaves a seam.
//
// A pause can be set on its own: kept as recorded, or retimed, faster or
// slower down to half speed. The squeeze levels the rest down together, the
// longest giving first, and leaves the pauses set by hand as they were set.

export type Cut = { end: number; id: string; start: number };

export type Piece = {
  end: number;
  isCut: boolean;
  /** The picture moves by itself: a page loading, a log, play. */
  isMoving: boolean;
  isSpeech: boolean;
  start: number;
  beat?: Beat;
};

/** A run of time between beats, as recorded. */
export type Pause = {
  end: number;
  id: string;
  /** Seconds of it where the picture moves and nothing is said. */
  moving: number;
  start: number;
  /** Seconds of it with nothing said and the picture still. */
  still: number;
};

/** How a pause is played: as recorded, or at its own length. A pause with
 * no setting is squeezed with the rest. */
export type PauseSetting = { kind: "keep" } | { kind: "own"; seconds: number };

export type TimeMap = {
  /** The edited recording's length, in seconds. */
  duration: number;
  /** The most a squeeze can take, in seconds. */
  maxSqueeze: number;
  /** How long a pause plays now, in seconds. */
  pauseLength: (pause: Pause) => number;
  pauses: Pause[];
  pieces: Piece[];
  /** The recording's own length, as recorded, in seconds. */
  recorded: number;
  /** The seconds the squeeze, at its present level, takes or would take
   * from a pause left to it. */
  squeezeTakes: (pause: Pause) => number;
  /** The recorded moment at a moment of the edited recording. */
  toOriginal: (time: number) => number;
  /** Where a recorded moment falls in the edited recording. */
  toTime: (seconds: number) => number;
};

/** The shortest a squeezed pause plays, in seconds. */
export const BREATH_S = 0.3;
/** Past this much still time, in seconds, a pause is dead air. */
export const DEAD_AIR_S = 4;

const contains = (span: { end: number; start: number }, at: number) =>
  at >= span.start && at < span.end;

function piecesOf({
  beats,
  cuts,
  isVoiced,
  recording,
}: {
  beats: Beat[];
  cuts: Cut[];
  isVoiced: boolean;
  recording: Recording;
}): Piece[] {
  const speech = isVoiced ? recording.speech : [];
  const spans = [...beats, ...speech, ...recording.picture, ...cuts];
  const edges = [
    ...new Set([
      0,
      recording.duration,
      ...spans.flatMap(({ end, start }) => [start, end]),
    ]),
  ]
    .filter((edge) => edge >= 0 && edge <= recording.duration)
    .sort((first, second) => first - second);
  return edges.slice(1).map((end, index) => {
    const start = edges[index];
    const middle = (start + end) / 2;
    const picture = recording.picture.find((span) => contains(span, middle));
    return {
      beat: beats.find((beat) => contains(beat, middle)),
      end,
      isCut: cuts.some((cut) => contains(cut, middle)),
      isMoving: (picture?.state ?? recording.rest) === "moving",
      isSpeech: speech.some((line) => contains(line, middle)),
      start,
    };
  });
}

const isGive = (piece: Piece) => !piece.beat && !piece.isSpeech && !piece.isCut;
const isStill = (piece: Piece) => isGive(piece) && !piece.isMoving;
const isMovingGive = (piece: Piece) => isGive(piece) && piece.isMoving;

/** The pauses, each a run of pieces between beats, named by where it
 * starts so a setting stays with it. */
function pausesOf(pieces: Piece[]): Pause[] {
  const pauses: Pause[] = [];
  for (const piece of pieces) {
    if (piece.beat || piece.isCut) continue;
    const length = piece.end - piece.start;
    const still = isStill(piece) ? length : 0;
    const moving = isMovingGive(piece) ? length : 0;
    const last = pauses[pauses.length - 1] as Pause | undefined;
    if (last && last.end === piece.start) {
      last.end = piece.end;
      last.still += still;
      last.moving += moving;
    } else
      pauses.push({
        end: piece.end,
        id: `pause-${String(piece.start)}`,
        moving,
        start: piece.start,
        still,
      });
  }
  return pauses;
}

export function timeMap({
  beats,
  cuts,
  isVoiced,
  recording,
  settings = {},
  squeeze,
}: {
  beats: Beat[];
  cuts: Cut[];
  isVoiced: boolean;
  recording: Recording;
  squeeze: number;
  /** Pauses set on their own, by id. */
  settings?: Record<string, PauseSetting>;
}): TimeMap {
  const pieces = piecesOf({ beats, cuts, isVoiced, recording });
  const pauses = pausesOf(pieces);
  const length = (piece: Piece) => piece.end - piece.start;
  const pauseOf = (piece: Piece) =>
    pauses.find(
      (pause) => piece.start >= pause.start && piece.end <= pause.end,
    );
  const settingOf = (piece: Piece) => {
    const pause = pauseOf(piece);
    return pause ? settings[pause.id] : undefined;
  };

  // Only still time in pauses nobody has set is levelled by the squeeze, no
  // faster than the picture allows.
  const cap = CAPS[recording.look];
  const free = pieces.filter((piece) => isStill(piece) && !settingOf(piece));
  const maxSqueeze = free.reduce(
    (sum, piece) => sum + length(piece) * (1 - 1 / cap.still),
    0,
  );
  const amount = Math.min(Math.max(squeeze, 0), maxSqueeze);
  const keptAt = (level: number, piece: Piece) =>
    Math.min(length(piece), Math.max(level, length(piece) / cap.still));
  let low = 0;
  let high = Math.max(0, ...free.map(length));
  for (let index = 0; index < 32 && amount > 0; index += 1) {
    const level = (low + high) / 2;
    const taken = free.reduce(
      (sum, piece) => sum + length(piece) - keptAt(level, piece),
      0,
    );
    if (taken > amount) low = level;
    else high = level;
  }

  const kept = pieces.map((piece) => {
    if (piece.isCut) return 0;
    if (!isGive(piece)) return length(piece);
    const setting = settingOf(piece);
    const pause = pauseOf(piece);
    if (setting?.kind === "keep") return length(piece);
    if (setting?.kind === "own" && pause) {
      // Its still and its moving give each share what is left of it once
      // what is said in it has played.
      const play = ownPlay(pause, setting.seconds, recording.look);
      return piece.isMoving
        ? length(piece) * (play.moving / pause.moving)
        : length(piece) * (play.still / pause.still);
    }
    if (piece.isMoving) return length(piece);
    return amount > 0 ? keptAt(high, piece) : length(piece);
  });
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
    if (index < 0) return recording.duration;
    const piece = pieces[index];
    const into = Math.max(0, time - starts[index]) / kept[index];
    return piece.start + into * length(piece);
  };
  return {
    duration,
    maxSqueeze,
    pauseLength: (pause) => toTime(pause.end) - toTime(pause.start),
    pauses,
    pieces,
    recorded: recording.duration,
    // At the squeeze's present level, whether the pause is levelled or not.
    squeezeTakes: (pause) =>
      amount > 0
        ? pieces
            .filter(
              (piece) =>
                isStill(piece) &&
                piece.start >= pause.start &&
                piece.end <= pause.end,
            )
            .reduce(
              (sum, piece) => sum + length(piece) - keptAt(high, piece),
              0,
            )
        : 0,
    toOriginal,
    toTime,
  };
}
