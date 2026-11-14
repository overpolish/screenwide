// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { MusicFilters } from "../../../../bindings/MusicFilters";
import type { MusicMood } from "../../../../bindings/MusicMood";
import type { MusicTempo } from "../../../../bindings/MusicTempo";
import type { MusicVocals } from "../../../../bindings/MusicVocals";

// What songs can be narrowed by, in the library and online, each filter's
// choices in the order its menu lists them, and their names.

export type { MusicFilters };

export type MusicFilterName = keyof MusicFilters;

type Choices<T extends string> = {
  /** The menu's choice that narrows nothing. */
  any: () => string;
  choices: readonly T[];
  label: (choice: T) => string;
  /** What the filter is, shown while it narrows nothing. */
  name: () => string;
};

const TEMPO: Choices<MusicTempo> = {
  any: () => t("editor-island-music-filter-tempo-any"),
  choices: ["slow", "medium", "fast"],
  label: (tempo) =>
    ({
      fast: () => t("editor-island-music-filter-tempo-fast"),
      medium: () => t("editor-island-music-filter-tempo-medium"),
      slow: () => t("editor-island-music-filter-tempo-slow"),
    })[tempo](),
  name: () => t("editor-island-music-filter-tempo"),
};

const VOCALS: Choices<MusicVocals> = {
  any: () => t("editor-island-music-filter-vocals-any"),
  choices: ["instrumental", "vocal"],
  label: (vocals) =>
    vocals === "instrumental"
      ? t("editor-island-music-filter-vocals-instrumental")
      : t("editor-island-music-filter-vocals-vocal"),
  name: () => t("editor-island-music-filter-vocals"),
};

const MOOD: Choices<MusicMood> = {
  any: () => t("editor-island-music-filter-mood-any"),
  choices: [
    "happy",
    "uplifting",
    "energetic",
    "calm",
    "dreamy",
    "romantic",
    "sad",
    "dark",
    "epic",
  ],
  label: (mood) =>
    ({
      calm: () => t("editor-island-music-filter-mood-calm"),
      dark: () => t("editor-island-music-filter-mood-dark"),
      dreamy: () => t("editor-island-music-filter-mood-dreamy"),
      energetic: () => t("editor-island-music-filter-mood-energetic"),
      epic: () => t("editor-island-music-filter-mood-epic"),
      happy: () => t("editor-island-music-filter-mood-happy"),
      romantic: () => t("editor-island-music-filter-mood-romantic"),
      sad: () => t("editor-island-music-filter-mood-sad"),
      uplifting: () => t("editor-island-music-filter-mood-uplifting"),
    })[mood](),
  name: () => t("editor-island-music-filter-mood"),
};

/** Each filter's choices and names. */
export const MUSIC_FILTERS = {
  mood: MOOD as Choices<string>,
  tempo: TEMPO as Choices<string>,
  vocals: VOCALS as Choices<string>,
} satisfies Record<MusicFilterName, Choices<string>>;

/** The filters in the order the line shows them. */
export const MUSIC_FILTER_ORDER: readonly MusicFilterName[] = [
  "tempo",
  "vocals",
  "mood",
];

export const NO_MUSIC_FILTERS: MusicFilters = {};

/** Each filter's choice, in the line's order. */
const valuesOf = (filters: MusicFilters) =>
  MUSIC_FILTER_ORDER.map((name) => filters[name]);

/** Whether `filters` narrows anything. */
export const hasMusicFilters = (filters: MusicFilters) =>
  valuesOf(filters).some((value) => value !== undefined);

/** `filters` as one string, the same however it was built, to tell
 * searches apart. */
export const musicFiltersKey = (filters: MusicFilters) =>
  valuesOf(filters).join("\n");
