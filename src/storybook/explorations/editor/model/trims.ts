// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Recording } from "../recording";

import { BREATH_S, type Cut, type Pause, type Piece } from "./timeline-model";

// Trimming a pause takes time away, so it is a cut, with a seam to put it
// back, never a tighter spring. A pause's end is trimmed back by dragging it
// with ⌥ held; "Trim to a breath" cuts all its still time but a breath; ⌥
// dragged across the strip cuts any stretch of it.
//
// A cut never splits a word. Its edges move to the nearest gap between
// words: tools that cut by the transcript alone clip the ends of words, the
// complaint their users raise most.

/** The least of a pause a trim leaves, in seconds. */
const LEAST_LEFT_S = 0.1;

/** Where a cut's edge goes so it splits no word: `at` itself outside
 * speech, or the gap between words nearest it, or the nearest one before
 * it for `"down"` or after it for `"up"`. `voice` is the recording when its
 * voice is heard, or null. */
export function wordEdge(
  at: number,
  voice: Recording | null,
  round: "down" | "nearest" | "up" = "nearest",
) {
  const line = voice?.speech.find(({ end, start }) => at > start && at < end);
  if (!voice || !line) return at;
  const edges = [
    line.start,
    ...voice.words
      .filter(({ at: start }) => start > line.start && start < line.end)
      .map(({ at: start }) => start),
    line.end,
  ].filter(
    (edge) =>
      round === "nearest" || (round === "down" ? edge <= at : edge >= at),
  );
  return edges.reduce((best, edge) =>
    Math.abs(edge - at) < Math.abs(best - at) ? edge : best,
  );
}

const trimId = (pause: Pause) => `trim-${pause.id}`;

/** The cuts with a pause's end trimmed back to `to`, in recorded seconds.
 * Drawn back out to where it first ended, the trim is let go. */
export function trimmedTo(
  cuts: Cut[],
  pause: Pause,
  { to, voice }: { to: number; voice: Recording | null },
): Cut[] {
  const trim = cuts.find(({ id }) => id === trimId(pause));
  const rest = cuts.filter(({ id }) => id !== trimId(pause));
  const end = trim?.end ?? wordEdge(pause.end, voice, "down");
  const least = pause.start + LEAST_LEFT_S;
  const nearest = wordEdge(Math.max(least, Math.min(to, end)), voice);
  const start = nearest < least ? wordEdge(least, voice, "up") : nearest;
  return end - start < 0.05
    ? rest
    : [...rest, { end, id: trimId(pause), start }];
}

/** The cuts with a pause's still time trimmed to a breath: all of it but
 * its first breath, leaving what is said or changes on screen in it whole.
 * Trimmed again, the breath's last cuts are replaced. */
export function breathed(cuts: Cut[], pause: Pause, pieces: Piece[]): Cut[] {
  const rest = cuts.filter(({ id }) => !id.startsWith(`breath-${pause.id}-`));
  return [...rest, ...breathCuts(pause, pieces)];
}

function breathCuts(pause: Pause, pieces: Piece[]): Cut[] {
  let breath = BREATH_S;
  return pieces
    .filter(
      (piece) =>
        piece.start >= pause.start &&
        piece.end <= pause.end &&
        !piece.beat &&
        !piece.isSpeech &&
        !piece.isMoving &&
        !piece.isCut,
    )
    .flatMap((piece, index) => {
      const kept = Math.min(breath, piece.end - piece.start);
      breath -= kept;
      const start = piece.start + kept;
      return piece.end - start < 0.05
        ? []
        : [
            {
              end: piece.end,
              id: `breath-${pause.id}-${String(index)}`,
              start,
            },
          ];
    });
}
