// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { DictationIslandState } from "../../../bindings/DictationIslandState";
import type { DictationWordChange } from "../../../bindings/DictationWordChange";
import type {
  DictationCommand,
  DictationCommandKind,
} from "../../transcription/types";

// Dictation in Settings' picture, each row's a timeline read from one
// clock, as the island shows it while someone talks: what is still being
// heard in grey, settling into the draft, and a command, once its pause has
// passed, carried out on the draft in its place. Each loops after a beat
// of the finished draft.

/** The rows of Dictation that each have a picture. */
export const DICTATION_SUBJECTS = [
  "scratch",
  "newLine",
  "newParagraph",
  "endList",
  "punctuation",
  "addWord",
  "removeWord",
] as const;

export type DictationSubject = (typeof DICTATION_SUBJECTS)[number];

export const isDictationSubject = (
  subject: string,
): subject is DictationSubject =>
  (DICTATION_SUBJECTS as readonly string[]).includes(subject);

/** A moment of the draft: from `at` seconds, what has settled, what is
 * still being heard, and the words added or removed so far. */
type Step = {
  at: number;
  settled: string;
  unsettled: string;
  taught?: DictationWordChange[];
};

/** One pass of a script: its steps, then how long it holds the last. */
type Script = { hold: number; steps: Step[] };

/** How long apart words arrive as someone talks, in seconds. */
const WORD_GAP = 0.28;
/** How many of the latest words are still being heard. */
const HEARING = 2;

/** `sentence` said a word at a time from `from`, settling as more come. */
function said(sentence: string, from: number, before = ""): Step[] {
  const words = sentence.split(" ");
  return words.map((_, index) => {
    const settledCount = Math.max(0, index + 1 - HEARING);
    const head = words.slice(0, settledCount).join(" ");
    return {
      at: from + index * WORD_GAP,
      settled: before + (head && `${head} `),
      unsettled: words.slice(settledCount, index + 1).join(" "),
    };
  });
}

/** `text` said from `from` and settled whole once its pause passes. */
function sentence(text: string, from: number, before = ""): Step[] {
  const steps = said(text, from, before);
  const end = from + steps.length * WORD_GAP + 0.4;
  return [...steps, { at: end, settled: before + text, unsettled: "" }];
}

/** A command heard after `draft` from `from`, then carried out as
 * `after`. */
function command(
  [draft, phrase]: [draft: string, phrase: string],
  from: number,
  after: string,
): Step[] {
  return [
    { at: from, settled: draft, unsettled: phrase },
    { at: from + 1.1, settled: after, unsettled: "" },
  ];
}

/** The phrase that does `does` in `commands`, the one typing `mark` among
 * punctuation, as the island would write it: capitalised. */
function phrase(
  commands: DictationCommand[],
  does: DictationCommandKind,
  mark?: string,
): string | undefined {
  const said = commands.find(
    (command) =>
      command.does === does && (mark === undefined || command.mark === mark),
  )?.said;
  return said && said.charAt(0).toLocaleUpperCase() + said.slice(1);
}

function script(
  subject: DictationSubject,
  commands: DictationCommand[],
): Script {
  switch (subject) {
    case "scratch": {
      const before = t("settings-dictation-sample-before");
      return {
        hold: 2,
        steps: [
          ...sentence(before, 0),
          ...command(
            [`${before} `, phrase(commands, "scratchThat") ?? ""],
            2,
            "",
          ),
          ...sentence(t("settings-dictation-sample-after"), 3.6),
        ],
      };
    }
    case "newLine":
    case "newParagraph": {
      const greeting = t("settings-dictation-sample-greeting");
      const broken = greeting + (subject === "newLine" ? "\n" : "\n\n");
      return {
        hold: 2,
        steps: [
          ...sentence(greeting, 0),
          ...command(
            [`${greeting} `, phrase(commands, subject) ?? ""],
            1.4,
            broken,
          ),
          ...sentence(t("settings-dictation-sample-thanks"), 3, broken),
        ],
      };
    }
    case "endList": {
      const list = `1. ${t("settings-dictation-sample-list-first")}\n2. ${t("settings-dictation-sample-list-second")}`;
      const ended = `${list}\n\n`;
      return {
        hold: 2,
        steps: [
          ...sentence(list, 0),
          ...command(
            [`${list} `, phrase(commands, "endList") ?? ""],
            1.6,
            ended,
          ),
          ...sentence(t("settings-dictation-sample-thanks"), 3.2, ended),
        ],
      };
    }
    case "punctuation": {
      const question = t("settings-dictation-sample-question");
      const mark = phrase(commands, "punctuation", "?") ? "?" : ".";
      return {
        hold: 2,
        steps: [
          ...sentence(question, 0),
          ...command(
            [`${question} `, phrase(commands, "punctuation", mark) ?? ""],
            2.6,
            `${question}${mark}`,
          ),
        ],
      };
    }
    case "addWord":
    case "removeWord": {
      const isAdding = subject === "addWord";
      const draft = isAdding
        ? t("settings-dictation-sample-word")
        : t("settings-dictation-sample-after");
      const said = phrase(commands, subject);
      // Without a command to say, only the word written as listed.
      if (!said) return { hold: 2.6, steps: sentence(draft, 0) };
      const spelled = isAdding
        ? t("settings-dictation-sample-added-spelled")
        : t("settings-dictation-sample-removed-spelled");
      const word = isAdding
        ? t("settings-dictation-sample-added")
        : t("settings-dictation-sample-removed");
      const from = 2.6;
      const settled = `${draft} `;
      return {
        hold: 2.6,
        steps: [
          ...sentence(draft, 0),
          { at: from, settled, unsettled: said },
          { at: from + 0.7, settled, unsettled: `${said} ${spelled}` },
          {
            at: from + 2,
            settled: draft,
            taught: [{ outcome: isAdding ? "added" : "removed", word }],
            unsettled: "",
          },
        ],
      };
    }
  }
}

/** What the island shows `time` seconds into `subject`'s loop, its
 * commands said as `commands` has them. */
export function dictationFrame(
  subject: DictationSubject,
  time: number,
  commands: DictationCommand[],
): DictationIslandState {
  const { hold, steps } = script(subject, commands);
  const last = steps[steps.length - 1];
  const at = time % (last.at + hold);
  const reached = steps.filter((each) => each.at <= at);
  const step = reached.length > 0 ? reached[reached.length - 1] : steps[0];
  return {
    atPauses: false,
    phase: "listening",
    settled: step.settled,
    taught: step.taught ?? [],
    unsettled: step.unsettled,
  };
}
