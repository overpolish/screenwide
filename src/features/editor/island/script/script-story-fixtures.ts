// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { captionText } from "../../tool-panels/captions/caption-text";
import { breakIn } from "../../tool-panels/captions/caption-word-choices";

import type { Caption } from "../../../../bindings/Caption";
import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { TranscriptCorrection } from "../../tool-panels/tool-panel-transcript";
import type { TranscriptWord } from "../../tool-panels/transcript/transcript-words";
import type { TranscriptMark } from "../../tool-panels/transcript/use-transcript-marks";

// Takes for the Script island's stories, and captions made of them as the
// app makes them: so many words to a caption, a break or a join placed by
// hand kept, hidden words left out and a key word set apart. The app's
// captions are made by `screenwide_text`; this stands in for it in a story,
// where there is no recording to make them from.

/** A take: its words, how long the recording runs, and the fillers and
 * profanity the Transcript panel would mark in it. */
export type StoryTake = {
  durationMs: number;
  marks: ReadonlyMap<number, TranscriptMark>;
  words: TranscriptWord[];
};

const bare = (text: string) => text.toLowerCase().replace(/[^\p{L}]/gu, "");

/** The take of `said`, a quarter second a word with a breath between, and
 * a longer pause after each sentence. */
function storyTake(said: readonly string[]): StoryTake {
  const words: TranscriptWord[] = [];
  let at = 600;
  for (const sentence of said) {
    for (const text of sentence.split(" ")) {
      words.push({ endMs: at + 260, probability: 0.92, startMs: at, text });
      at += 330;
    }
    at += 1_700;
  }
  const marks = new Map(
    words.flatMap((word, index): [number, TranscriptMark][] => {
      const heard = bare(word.text);
      if (heard === "um" || heard === "uh") return [[index, "filler"]];
      if (heard === "damn") return [[index, "profanity"]];
      return [];
    }),
  );
  return { durationMs: at + 1_000, marks, words };
}

const SAID = [
  "So, um, today I want to show you how the new editor works.",
  "First we open a recording, and the island comes up right where you clicked.",
  "Damn, that was quick. Uh, then you pick a template for the captions,",
  "and everything you say shows up here as a script you can fix.",
  "Double-click a word to correct it, or open its menu for the rest.",
];

export const SCRIPT_TAKE = storyTake(SAID);

/** A take long enough that the script scrolls within the island. */
export const LONG_SCRIPT_TAKE = storyTake([
  ...SAID,
  "Breaks you place by hand show in blue, so you can tell them apart.",
  "Um, the faint marks are where the captions end on their own.",
  "A long pause always starts a new caption, and so does a full stop.",
  "If two captions should read as one, join them where they meet.",
  "Hidden words stay in the recording, struck through in the script.",
  "Uh, fillers like this one can be hidden all at once from the Captions island.",
  "Key words are set apart in the template's accent, one to a caption.",
  "Pick a different one from the menu if the longest word is not the point.",
  "Spelling you fix here goes into the transcript, so every mode sees it.",
  "Reset caption edits takes it all back to what was heard.",
  "Damn, that is a lot to remember, but the hint is always at the foot.",
  "And when you are done, press Escape to close both islands.",
]);

const shareOf = (word: TranscriptWord, durationMs: number) => ({
  from: word.startMs / durationMs,
  to: word.endMs / durationMs,
});

/** The choice made by hand of `word`, if any. */
const choiceOf = <K extends string>(
  choices: readonly { kind: K; sourceAt: number }[] | undefined,
  word: TranscriptWord,
  durationMs: number,
) => {
  const { from, to } = shareOf(word, durationMs);
  return (
    choices?.find(({ sourceAt }) => sourceAt >= from && sourceAt <= to)?.kind ??
    null
  );
};

/** Captions made of the take as `captions` set them: shown words only, a
 * caption ending at its most words, a split or a sentence's pause, and
 * running on over a join; its longest word its key word where the template
 * sets one apart, but for words set apart or kept plain by hand. */
export function makeStoryCaptions(
  captions: RecordingCaptions,
  { durationMs, marks, words }: StoryTake,
): Caption[] {
  const { accent, maxWords } = captions.style;
  const breaks = captions.breaks ?? [];
  const made: TranscriptWord[][] = [];
  let current: TranscriptWord[] = [];
  words.forEach((word, index) => {
    const mark = marks.get(index);
    const choice = choiceOf(captions.wordChoices, word, durationMs);
    const ruledOut =
      (mark === "filler" && captions.settings.hideFillers) ||
      (mark === "profanity" &&
        captions.settings.censorProfanity &&
        captions.settings.profanity === "remove");
    if (choice === "hidden" || (ruledOut && choice !== "shown")) return;
    const before = words[index - 1] as TranscriptWord | undefined;
    const placed = before
      ? breakIn(breaks, { after: word, before, durationMs })?.kind
      : undefined;
    const pause = before ? word.startMs - before.endMs > 1_000 : false;
    const full = current.length >= maxWords;
    if (
      current.length > 0 &&
      placed !== "join" &&
      (placed === "split" || pause || full)
    ) {
      made.push(current);
      current = [];
    }
    current.push(word);
  });
  if (current.length > 0) made.push(current);
  return made.map((said) => {
    const longest = said.reduce((a, b) =>
      bare(b.text).length > bare(a.text).length ? b : a,
    );
    return {
      endMs: said[said.length - 1].endMs,
      id: said[0].startMs,
      lines: [
        said.map((heard) => {
          const chosen = choiceOf(captions.emphasisChoices, heard, durationMs);
          return {
            emphasis:
              accent === "word" &&
              (chosen === "emphasized" ||
                (chosen === null && heard === longest)),
            endMs: heard.endMs,
            sourceEndMs: heard.endMs,
            sourceStartMs: heard.startMs,
            startMs: heard.startMs,
            text: heard.text,
          };
        }),
      ],
      spaced: true,
      startMs: said[0].startMs,
      text: said.map((heard) => heard.text).join(" "),
    };
  });
}

/** `take` laid out as `captions` break it, with `corrections` taken. */
export function storyScriptText(
  captions: RecordingCaptions,
  corrections: readonly TranscriptCorrection[],
  { isMade = true, take }: { take: StoryTake; isMade?: boolean },
) {
  return captionText(
    take.words,
    {
      corrections,
      durationMs: take.durationMs,
      played: [],
      removed: [],
    },
    {
      breaks: captions.breaks ?? [],
      captions: isMade ? makeStoryCaptions(captions, take) : [],
      keyWords: captions.style.accent === "word",
    },
  );
}
