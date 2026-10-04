// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { StickerArt } from "../annotations/annotations";

/**
 * The pictures a sticker can show, as the editor names and lists them.
 *
 * A sticker's emoji is stored as `emoji:` and the emoji itself, and drawn by
 * the system's own emoji font, in the picker and on the picture alike. The
 * list the picker shows and searches is `emoji.json`, written by
 * `scripts/prepare-emoji.mjs` from Unicode's and CLDR's data.
 */

/** The picture a sticker shows before one has been chosen: a thumbs up. The
 * twin of `DEFAULT_STICKER_ASSET` in
 * `src-tauri/src/editor/annotations/sticker/model.rs`. */
export const DEFAULT_STICKER: StickerArt = { aspect: 1, asset: "emoji:👍" };

type StickerEmoji = { emoji: string; keywords: string[]; name: string };
export type StickerEmojiGroup = { emoji: StickerEmoji[]; group: string };

/** The emoji in their groups, read when the picker first asks. */
export const loadStickerEmoji = () =>
  import("./emoji.json").then(
    ({ default: groups }) => groups as StickerEmojiGroup[],
  );

/** The asset id an emoji is stored as. */
export const emojiAsset = (emoji: string) => `emoji:${emoji}`;

/** The emoji in `groups` whose name or keywords take in every word of
 * `query`, in their groups, leaving out the groups with none. */
export const matchingStickerEmoji = (
  groups: StickerEmojiGroup[],
  query: string,
): StickerEmojiGroup[] => {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return groups;
  const matches = (emoji: StickerEmoji) =>
    words.every(
      (word) =>
        emoji.name.includes(word) ||
        emoji.keywords.some((keyword) => keyword.includes(word)),
    );
  return groups
    .map(({ emoji, group }) => ({ emoji: emoji.filter(matches), group }))
    .filter(({ emoji }) => emoji.length > 0);
};
