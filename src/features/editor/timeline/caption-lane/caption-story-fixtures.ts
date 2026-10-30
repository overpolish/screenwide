// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineCaption } from "./caption-line";

/** A line said from `start` to `end`, its words evenly through it. */
export const storyCaption = (
  start: number,
  end: number,
  text: string,
): TimelineCaption => {
  const words = text.split(" ");
  return {
    end,
    start,
    text,
    words: words.map((word, index) => ({
      at: start + ((end - start) * index) / words.length,
      word,
    })),
  };
};
