// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What kind of music a song is, as Openverse names it, after its title, so
// it can be judged before it is heard: the first of its genres, kept whole
// while the title is cut short. A song added from a file has none.

export function SongGenre({ genres }: { genres: readonly string[] }) {
  return genres.length === 0 ? null : (
    <span className="shrink-0 text-caption whitespace-nowrap text-fg-tertiary">
      {genres[0]}
    </span>
  );
}
