// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// How the stand-in online search answers in browser Storybook, which has no
// app to ask (`.storybook/preview.ts`). A Library story sets it before it
// renders, from `beforeEach`, which puts it back to finding something after.

export type StoryOnlineSearch =
  /** Songs or sound effects, one under each licence. */
  | "found"
  /** Searching still, with no answer coming. */
  | "searching"
  | "nothing"
  /** Too many searches this minute. */
  | "busy"
  /** Today's searches used up, until the end of the day. */
  | "usedUp"
  | "unavailable"
  /** Found, but each download refused as no longer offered under a licence
   * the app may use. */
  | "withdrawn";

let answer: StoryOnlineSearch = "found";

export const storyOnlineSearch = () => answer;

/** Answers the story's online searches as `next`, and gives back what puts
 * them back. */
export const answerOnlineSearch = (next: StoryOnlineSearch) => {
  answer = next;
  return () => {
    answer = "found";
  };
};
