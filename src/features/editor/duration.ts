// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * One frame of the recording preview. The native player renders at
 * `PREVIEW_FPS` (src-tauri/src/editor/recording_preview_player/video.rs), which
 * it never reports to the frontend, so keyboard stepping mirrors it here.
 */
export const PREVIEW_FRAME_MS = 1_000 / 30;

/** `01:04:07`, or `04:07` for anything under an hour. */
export const formatDuration = (durationMs: number) => {
  const total = Math.max(0, Math.floor(durationMs / 1000));
  const seconds = String(total % 60).padStart(2, "0");
  const minutes = Math.floor(total / 60) % 60;
  const hours = Math.floor(total / 3600);

  return hours > 0
    ? `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${seconds}`
    : `${String(minutes).padStart(2, "0")}:${seconds}`;
};

/**
 * A deliberately coarse remaining-time phrase for a running export, e.g.
 * `About 2 min remaining`, `About 40 sec remaining`, or
 * `Less than 10 sec remaining`. Rounds hard so the estimate never implies
 * precision it does not have.
 */
export const formatEta = (seconds: number) => {
  let total = Math.max(0, Math.round(seconds));
  // Below a minute the estimate is too jittery to put a number on, and the
  // remaining wait is short enough that a single steady phrase reads better
  // than a second-by-second countdown.
  if (total < 60) return "Less than a minute remaining";

  const days = Math.floor(total / 86_400);
  total -= days * 86_400;
  let hours = Math.floor(total / 3_600);
  total -= hours * 3_600;
  let minutes = Math.round(total / 60);
  // Rounding can carry a component to its own ceiling (59.6s → 60 min); roll it
  // up so "1 hr 60 min" never prints.
  if (minutes === 60) {
    minutes = 0;
    hours += 1;
  }
  let allDays = days;
  if (hours === 24) {
    hours = 0;
    allDays += 1;
  }

  const parts: string[] = [];
  if (allDays > 0)
    parts.push(`${String(allDays)} day${allDays === 1 ? "" : "s"}`);
  if (hours > 0) parts.push(`${String(hours)} hr`);
  // Always show minutes unless a larger unit already carries the estimate on
  // its own (e.g. exactly "1 hr remaining").
  if (minutes > 0 || parts.length === 0) parts.push(`${String(minutes)} min`);
  return `About ${parts.join(" ")} remaining`;
};

const SIZE_UNITS = ["bytes", "KB", "MB", "GB", "TB"];
/** Finder's file style: whole kilobytes, then up to one more decimal per
 * unit, without trailing zeros, in the reader's own number format. */
const SIZE_DECIMALS = [0, 0, 1, 2, 2];

/** How many bytes make the next unit up: Finder counts in thousands, as macOS
 * has since 10.6, and Explorer in 1024s, so a size reads as each shows it. */
const sizeBase = () =>
  document.documentElement.dataset.platform === "windows" ? 1024 : 1000;

export const formatBytes = (bytes: number, base = sizeBase()) => {
  if (bytes <= 0) return "Unknown size";
  let order = Math.min(
    Math.floor(Math.log(bytes) / Math.log(base)),
    SIZE_UNITS.length - 1,
  );
  // A size just short of the next unit can round up to it, as 999,999 bytes
  // does to 1,000 KB; it reads as the next unit instead.
  const rounded = (unit: number) =>
    Number((bytes / base ** unit).toFixed(SIZE_DECIMALS[unit]));
  if (rounded(order) >= base && order < SIZE_UNITS.length - 1) order += 1;
  return `${new Intl.NumberFormat(undefined, {
    maximumFractionDigits: SIZE_DECIMALS[order],
  }).format(rounded(order))} ${SIZE_UNITS[order]}`;
};
