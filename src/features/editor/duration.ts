// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { appLocale, t } from "../../i18n/i18n";
import { isWindows } from "../../lib/platform";

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
 * `About 2 min remaining` or `Less than a minute remaining`. Rounds hard so
 * the estimate never implies precision it does not have.
 */
export const formatEta = (seconds: number) => {
  let total = Math.max(0, Math.round(seconds));
  // Below a minute the estimate is too jittery to put a number on, and the
  // remaining wait is short enough that a single steady phrase reads better
  // than a second-by-second countdown.
  if (total < 60) return t("editor-eta-under-minute");

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
  if (allDays > 0) parts.push(t("editor-eta-days", { count: allDays }));
  if (hours > 0) parts.push(t("editor-eta-hours", { count: hours }));
  // Always show minutes unless a larger unit already carries the estimate on
  // its own (e.g. exactly "1 hr remaining").
  if (minutes > 0 || parts.length === 0)
    parts.push(t("editor-eta-minutes", { count: minutes }));
  const duration = new Intl.ListFormat(appLocale().formatLocale, {
    style: "narrow",
    type: "unit",
  }).format(parts);
  return t("editor-eta", { duration });
};

const SIZE_UNITS = [
  "byte",
  "kilobyte",
  "megabyte",
  "gigabyte",
  "terabyte",
] as const;
/** Finder's file style: whole kilobytes, then up to one more decimal per
 * unit, without trailing zeros, in the reader's own number format. */
const SIZE_DECIMALS = [0, 0, 1, 2, 2];

/** How many bytes make the next unit up: Finder counts in thousands, as macOS
 * has since 10.6, and Explorer in 1024s, so a size reads as each shows it. */
const sizeBase = () => (isWindows() ? 1024 : 1000);

type FormatBytesOptions = {
  base?: number;
  /** Always this many decimals, trailing zeros kept, so a size that is still
   * growing changes width only when it gains a digit or a unit: 67.0 MB, not
   * 67 MB. Without it, Finder's decimals. */
  decimals?: number;
  /** The smallest unit written, so a growing size does not hop between
   * units: 100 KB as 0.1 MB. */
  smallestUnit?: (typeof SIZE_UNITS)[number];
};

export const formatBytes = (
  bytes: number,
  {
    base = sizeBase(),
    decimals,
    smallestUnit = "byte",
  }: FormatBytesOptions = {},
) => {
  if (bytes <= 0) return t("format-size-unknown");
  const largest = SIZE_UNITS.length - 1;
  let order = Math.min(
    Math.max(
      Math.floor(Math.log(bytes) / Math.log(base)),
      SIZE_UNITS.indexOf(smallestUnit),
    ),
    largest,
  );
  const places = (unit: number) => decimals ?? SIZE_DECIMALS[unit];
  // A size just short of the next unit can round up to it, as 999,999 bytes
  // does to 1,000 KB; it reads as the next unit instead.
  const rounded = (unit: number) =>
    Number((bytes / base ** unit).toFixed(places(unit)));
  if (rounded(order) >= base && order < largest) order += 1;
  const size = new Intl.NumberFormat(appLocale().formatLocale, {
    maximumFractionDigits: places(order),
    minimumFractionDigits: decimals ?? 0,
  }).format(rounded(order));
  return t("format-size", { size, unit: SIZE_UNITS[order] });
};
