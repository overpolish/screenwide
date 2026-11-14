// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What music can be narrowed by: how fast it is, whether it is sung, and how
// it feels. Openverse has no fields for these; Jamendo, its music source,
// tags each track with them, and Openverse's search requires every word it
// is given, so each filter is a tag added to the words searched. Each mood
// is the one of Jamendo's tags that finds most for it.

const NAMES = ["tempo", "vocals", "mood"] as const;

type FilterName = (typeof NAMES)[number];

/** Each filter's choices, as the app names them, and the tag each stands
 * for. */
const TAGS: Record<FilterName, Partial<Record<string, string>>> = {
  mood: {
    calm: "peaceful",
    dark: "dark",
    dreamy: "dreamy",
    energetic: "energetic",
    epic: "epic",
    happy: "happy",
    romantic: "romantic",
    sad: "sad",
    uplifting: "uplifting",
  },
  tempo: { fast: "speed_high", medium: "speed_medium", slow: "speed_low" },
  vocals: { instrumental: "instrumental", vocal: "vocal" },
};

/** The choice made for each filter set, by the app's names. */
export type MusicFilters = Partial<Record<FilterName, string>>;

/** The filters `params` asks for, or none where one names no choice. */
export function parseMusicFilters(
  params: URLSearchParams,
): MusicFilters | null {
  const filters: MusicFilters = {};
  for (const name of NAMES) {
    const value = params.get(name);
    if (value === null || value === "") continue;
    if (!Object.hasOwn(TAGS[name], value)) return null;
    filters[name] = value;
  }
  return filters;
}

/** Whether `filters` narrows anything. */
export const hasMusicFilters = (filters: MusicFilters) =>
  NAMES.some((name) => filters[name] !== undefined);

/** The tags `filters` requires. */
export const musicTags = (filters: MusicFilters) =>
  NAMES.flatMap((name) => {
    const value = filters[name];
    const tag = value === undefined ? undefined : TAGS[name][value];
    return tag === undefined ? [] : [tag];
  });

/** What a track is as music, in the app's words, as the library keeps it. */
export type MusicDescription = {
  genres: string[];
  moods: string[];
  tempo: string | null;
  vocals: string | null;
};

/** The choices of filter `name` whose tag is among `tags`. */
const chosenBy = (name: FilterName, tags: ReadonlySet<string>) =>
  Object.entries(TAGS[name]).flatMap(([choice, tag]) =>
    tag !== undefined && tags.has(tag) ? [choice] : [],
  );

/** Where a track has one of a filter's tags, that choice; none where it has
 * none, or more than one, which says nothing. */
const onlyChoice = (name: FilterName, tags: ReadonlySet<string>) => {
  const chosen = chosenBy(name, tags);
  return chosen.length === 1 ? (chosen[0] ?? null) : null;
};

/** A track described by its `tags` and `genres`, as Jamendo gives them. */
export const describeMusic = (
  tags: string[],
  genres: string[],
): MusicDescription => {
  const held = new Set(tags);
  return {
    genres,
    moods: chosenBy("mood", held),
    tempo: onlyChoice("tempo", held),
    vocals: onlyChoice("vocals", held),
  };
};
