// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { matchingStickerEmoji, StickerEmojiGroup } from "./sticker-library";

const groups: StickerEmojiGroup[] = [
  {
    emoji: [
      { emoji: "😀", keywords: ["face", "grin"], name: "grinning face" },
      { emoji: "🔥", keywords: ["flame", "hot"], name: "fire" },
    ],
    group: "Smileys & Emotion",
  },
  {
    emoji: [{ emoji: "👍", keywords: ["+1", "hand", "up"], name: "thumbs up" }],
    group: "People & Body",
  },
];

describe("matchingStickerEmoji", () => {
  it("finds an emoji by every word typed, in its name or its keywords", () => {
    expect(matchingStickerEmoji(groups, "HOT  fi")).toEqual([
      {
        emoji: [{ emoji: "🔥", keywords: ["flame", "hot"], name: "fire" }],
        group: "Smileys & Emotion",
      },
    ]);
    expect(matchingStickerEmoji(groups, "thumbs face")).toEqual([]);
  });

  it("lists everything while nothing is typed", () => {
    expect(matchingStickerEmoji(groups, "  ")).toBe(groups);
  });
});
