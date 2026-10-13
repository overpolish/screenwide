// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { captionSaidAt, captionText } from "./caption-text";
import { withBreak, withChoice } from "./caption-word-choices";

import type { Caption } from "../../../../bindings/Caption";
import type { TranscriptWord } from "../transcript/transcript-words";

const word = (text: string, startMs: number): TranscriptWord => ({
  endMs: startMs + 300,
  probability: 0.9,
  startMs,
  text,
});

/** A caption showing `said`, each word timed as the transcript heard it. */
const caption = (said: readonly TranscriptWord[]): Caption => ({
  endMs: said[said.length - 1].endMs,
  id: said[0].startMs,
  lines: [
    said.map((heard) => ({
      endMs: heard.endMs,
      sourceEndMs: heard.endMs,
      sourceStartMs: heard.startMs,
      startMs: heard.startMs,
      text: heard.text,
    })),
  ],
  spaced: true,
  startMs: said[0].startMs,
  text: said.map((heard) => heard.text).join(" "),
});

const untouched = {
  corrections: [],
  durationMs: 20_000,
  played: [],
  removed: [],
};

/** Each paragraph as text, a gap's mark before the word it comes before,
 * and a word the captions leave out in brackets. */
const shown = (text: ReturnType<typeof captionText>) =>
  text.paragraphs.map(({ end, start }) =>
    text.words
      .slice(start, end)
      .map((laid) => {
        const mark = { break: "| ", join: "~ ", none: "", split: "/ " }[
          laid.gap
        ];
        const said =
          text.isMarked && laid.caption === null ? `[${laid.text}]` : laid.text;
        return `${mark}${said}`;
      })
      .join(" "),
  );

describe("captionText", () => {
  // So um this is a test, then a long pause, then two words.
  const words = [
    word("So", 1_000),
    word("um", 1_400),
    word("this", 1_800),
    word("is", 2_200),
    word("a", 2_600),
    word("test.", 3_000),
    word("Next", 8_000),
    word("part.", 8_400),
  ];

  it("marks a break between captions and keeps a hidden word in place", () => {
    const captions = [
      caption([words[0], words[2]]),
      caption(words.slice(3, 6)),
      caption(words.slice(6)),
    ];
    expect(
      shown(
        captionText(words, untouched, { allCaps: false, breaks: [], captions }),
      ),
    ).toEqual(["So [um] this | is a test.", "| Next part."]);
  });

  it("shows corrected words and leaves out what the edit cuts", () => {
    const text = captionText(
      words,
      {
        ...untouched,
        corrections: [{ endMs: 2_500, startMs: 1_800, text: "this's" }],
        removed: [{ endMs: 1_750, startMs: 1_350 }],
      },
      {
        allCaps: false,
        breaks: [],
        captions: [caption([words[0]]), caption(words.slice(2, 8))],
      },
    );
    expect(shown(text)).toEqual(["So | this's a test.", "Next part."]);
  });

  it("shows breaks and joins placed by hand where they were placed", () => {
    const gap = (at: number) => ({
      after: words[at],
      before: words[at - 1],
      durationMs: 20_000,
    });
    let breaks = withBreak([], gap(4), "split");
    breaks = withBreak(breaks, gap(6), "join");
    const text = captionText(words, untouched, {
      allCaps: false,
      breaks,
      captions: [caption(words)],
    });
    expect(shown(text)).toEqual(["So um this is / a test.", "~ Next part."]);
    // Placing again in the same gap replaces what was there.
    expect(withBreak(breaks, gap(4), null)).toEqual([breaks[1]]);
  });

  it("marks nothing until the captions are made", () => {
    const text = captionText(words, untouched, {
      allCaps: false,
      breaks: [],
      captions: [],
    });
    expect(text.isMarked).toBe(false);
    expect(shown(text)).toEqual(["So um this is a test.", "Next part."]);
  });

  it("finds the caption being said", () => {
    const { captions } = captionText(words, untouched, {
      allCaps: false,
      breaks: [],
      captions: [caption(words.slice(0, 3)), caption(words.slice(3, 6))],
    });
    expect(captionSaidAt(captions, 1_900)).toBe(0);
    expect(captionSaidAt(captions, 3_100)).toBe(1);
    expect(captionSaidAt(captions, 5_000)).toBeNull();
  });

  it("reads each word as its caption sets it, in the case it was said", () => {
    const said = [word("Oh", 1_000), word("damn,", 1_400), word("Bob.", 1_800)];
    const set = caption(said);
    set.lines[0][0].text = "OH";
    set.lines[0][1].text = "D***";
    set.lines[0][2].text = "BOB";
    const text = captionText(said, untouched, {
      allCaps: true,
      breaks: [],
      captions: [set],
    });
    expect(text.words.map((laid) => laid.captionText)).toEqual([
      "Oh",
      "d***",
      "Bob",
    ]);
  });

  it("reads a word as corrected until its caption catches up", () => {
    const said = [word("gonna", 1_000)];
    const text = captionText(
      said,
      {
        ...untouched,
        corrections: [{ endMs: 1_300, startMs: 1_000, text: "going to" }],
      },
      { allCaps: false, breaks: [], captions: [caption(said)] },
    );
    expect(text.words[0]).toMatchObject({
      captionText: null,
      corrected: true,
      text: "going to",
    });
  });
});

describe("withChoice", () => {
  it("keeps one choice a word, at its middle", () => {
    const um = { durationMs: 20_000, word: word("um", 1_400) };
    const hidden = withChoice([], um, "hidden");
    expect(hidden).toEqual([{ kind: "hidden", sourceAt: 0.0775 }]);
    expect(withChoice(hidden, um, "shown")).toEqual([
      { kind: "shown", sourceAt: 0.0775 },
    ]);
    expect(withChoice(hidden, um, null)).toEqual([]);
  });
});
