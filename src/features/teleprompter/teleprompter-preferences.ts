// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { create } from "zustand";

// How the teleprompter reads, kept in its window's storage for next time.

export type TeleprompterMode = "steady" | "voice";

export type TeleprompterPreferences = {
  /** The widest the text runs, in points, centred in the window; null lets
   * it fill the window. */
  columnWidth: number | null;
  isMirrored: boolean;
  mode: TeleprompterMode;
  /** The text's size, in points. */
  textSize: number;
  /** The steady pace, in words per minute. */
  wordsPerMinute: number;
};

/** The fields step from their minimum, so each default is on the grid. */
export const TEXT_SIZES = { max: 96, min: 20, step: 4 } as const;
export const PACES = { max: 300, min: 60, step: 10 } as const;
/** How narrow and wide the column may be set, in points. */
export const COLUMN_WIDTHS = { max: 8192, min: 240 } as const;

const DEFAULTS: TeleprompterPreferences = {
  columnWidth: null,
  isMirrored: false,
  mode: "voice",
  textSize: 36,
  wordsPerMinute: 140,
};

const KEY = "screenwide.teleprompter";

const clamp = (value: unknown, { max, min }: { max: number; min: number }) =>
  typeof value === "number" && Number.isFinite(value)
    ? Math.min(max, Math.max(min, value))
    : null;

const stored = (): TeleprompterPreferences => {
  try {
    const parsed: unknown = JSON.parse(
      window.localStorage.getItem(KEY) ?? "null",
    );
    if (typeof parsed !== "object" || parsed === null) return DEFAULTS;
    return {
      columnWidth:
        "columnWidth" in parsed
          ? clamp(parsed.columnWidth, COLUMN_WIDTHS)
          : DEFAULTS.columnWidth,
      isMirrored:
        "isMirrored" in parsed && typeof parsed.isMirrored === "boolean"
          ? parsed.isMirrored
          : DEFAULTS.isMirrored,
      mode:
        "mode" in parsed &&
        (parsed.mode === "steady" || parsed.mode === "voice")
          ? parsed.mode
          : DEFAULTS.mode,
      textSize:
        ("textSize" in parsed ? clamp(parsed.textSize, TEXT_SIZES) : null) ??
        DEFAULTS.textSize,
      wordsPerMinute:
        ("wordsPerMinute" in parsed
          ? clamp(parsed.wordsPerMinute, PACES)
          : null) ?? DEFAULTS.wordsPerMinute,
    };
  } catch {
    return DEFAULTS;
  }
};

export const useTeleprompterPreferences = create<
  TeleprompterPreferences & {
    change: (patch: Partial<TeleprompterPreferences>) => void;
  }
>((set, get) => ({
  ...stored(),
  change: (patch) => {
    set(patch);
    const { change: _change, ...preferences } = get();
    try {
      window.localStorage.setItem(KEY, JSON.stringify(preferences));
    } catch {
      // The preference stands for this session even where it cannot be kept.
    }
  },
}));

/** `value` moved `steps` steps, held within `range`. */
export const stepped = (
  value: number,
  steps: number,
  range: { max: number; min: number; step: number },
) => Math.min(range.max, Math.max(range.min, value + steps * range.step));
