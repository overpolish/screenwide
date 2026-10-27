// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type PictureSpan } from "../recording";

// What a recording's picture readings say it was doing, for the readings
// the capture writes beside a movie (`<movie>-picture.jsonl`). The same
// judgement as `states` in `src-tauri/crates/activity/src/states.rs`, which
// is where the app makes it; repeated here only so Storybook can show a
// recording's own readings without the app.

export type PictureReading = { changed: number; ms: number; novelty: number };
type State = PictureSpan["state"];

const MOVING_SHARE = 0.002;
const NEW_SHARE = 0.25;
const BUSY_NOVELTY = 0.05;
const NEW_RARITY = 0.03;
const AROUND = 100;
const SETTLE = 10;
const CALM_SHARE = 0.05;

const quantile = (values: number[], share: number) => {
  const sorted = [...values].sort((first, second) => first - second);
  return sorted[Math.round((sorted.length - 1) * share)] ?? 0;
};

function states(readings: PictureReading[]): State[] {
  const changed = readings.map((reading) => reading.changed);
  const novelty = readings.map((reading) => reading.novelty);
  const moving = Math.max(MOVING_SHARE, 0.5 * quantile(changed, 0.5));
  const fresh =
    quantile(novelty, 0.5) > BUSY_NOVELTY
      ? Math.max(NEW_SHARE, quantile(novelty, 1 - NEW_RARITY))
      : NEW_SHARE;
  return readings.map((reading, index) => {
    const around = quantile(
      novelty.slice(Math.max(0, index - AROUND / 2), index + AROUND / 2),
      0.5,
    );
    const next = changed.slice(index + 1, index + 1 + SETTLE);
    const settles =
      next.reduce((sum, value) => sum + value, 0) / Math.max(next.length, 1) <=
      Math.max(CALM_SHARE, moving);
    if (reading.novelty >= fresh && reading.novelty >= 2 * around && settles)
      return "new";
    return reading.changed >= moving ? "moving" : "still";
  });
}

/** The stretches where the picture moves or is new, in seconds; the rest
 * is still. Each reading stands for the tenth of a second it was taken. */
export function pictureSpans(readings: PictureReading[]): PictureSpan[] {
  const found = states(readings);
  const spans: PictureSpan[] = [];
  readings.forEach((reading, index) => {
    const state = found[index];
    const start = reading.ms / 1000;
    const last = spans[spans.length - 1] as PictureSpan | undefined;
    if (state === "still") return;
    if (last && last.state === state && Math.abs(last.end - start) < 1e-6)
      last.end = start + 0.1;
    else spans.push({ end: start + 0.1, start, state });
  });
  return spans;
}

/** The readings in a `-picture.jsonl` file's text. */
export const readingsOf = (text: string): PictureReading[] =>
  text
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as { type: string } & PictureReading)
    .filter(({ type }) => type === "reading");
