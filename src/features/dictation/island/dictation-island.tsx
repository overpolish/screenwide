// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic, X } from "lucide-react";
import { useState } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { Alert } from "../../../primitives/alert/alert";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { Island } from "../../../primitives/island/island";
import { IslandKeyHint } from "../../../primitives/island/island-key-hint";
import { IslandParts } from "../../../primitives/island/island-parts";
import { type IslandProps } from "../../../primitives/island/island-props";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";
import { Text } from "../../../primitives/text/text";
import { LevelFill } from "../../recording-controls/island/island-glyphs";

import type { DictationIslandAction } from "../../../bindings/DictationIslandAction";
import type { DictationIslandState } from "../../../bindings/DictationIslandState";
import type { DictationProblem } from "../../../bindings/DictationProblem";
import type { DictationWordChange } from "../../../bindings/DictationWordChange";

// Dictation on the island, at the top of the screen. Listening, its row
// names it beside the microphone's glyph, which fills with its level, with
// Cancel and Done at its end; the words hang under it as they are heard, the
// ones still being heard dimmer, and before any a hint says how to cancel.
// Done, the row says the words are going in, and the words stay as they
// were. A problem says why under the row, with Settings where it is put
// right.

/** The width the words wrap at, so the island does not grow word by word. */
const WORDS_WIDTH = "w-96";

export type DictationIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
> & {
  /** The microphone's level, 0 to 1. */
  level: number;
  onAction: (action: DictationIslandAction) => void;
  state: DictationIslandState;
};

type Words = { settled: string; unsettled: string };

const problemMessage = (problem: DictationProblem) => {
  switch (problem) {
    case "noModel":
      return t("dictation-island-no-model");
    case "noMicrophone":
      return t("dictation-island-no-microphone");
    case "noAccessibility":
      return t("dictation-island-no-accessibility");
    case "passwordField":
      return t("dictation-island-password-field");
    case "administrator":
      return t("dictation-island-administrator");
    case "failed":
      return t("dictation-island-failed");
  }
};

/** Whether Settings is where `problem` is put right. */
const isFixedInSettings = (problem: DictationProblem) =>
  problem === "noModel" ||
  problem === "noMicrophone" ||
  problem === "noAccessibility";

function Close({ label, onPress }: { label: string; onPress: () => void }) {
  return (
    <IconButton
      aria-label={label}
      className="text-fg-secondary enabled:hover:text-fg"
      onPress={onPress}
    >
      <X aria-hidden className="size-icon" />
    </IconButton>
  );
}

/** The words heard so far, their latest lines in view; before any, how to
 * dictate, which says the text comes at each pause where it does. */
function HeardWords({ atPauses, words }: { atPauses: boolean; words: Words }) {
  if (!words.settled && !words.unsettled)
    return (
      <IslandKeyHint
        binding="Escape"
        className={WORDS_WIDTH}
        message={(key) =>
          atPauses
            ? t("dictation-island-hint-at-pauses", { key })
            : t("dictation-island-hint", { key })
        }
      />
    );
  return (
    <div
      aria-live="polite"
      className={cn(
        "flex max-h-24 flex-col justify-end overflow-hidden px-inset-sm",
        WORDS_WIDTH,
      )}
    >
      <Text as="p" className="whitespace-pre-wrap" variant="body">
        {words.settled}
        <span className="text-fg-secondary">{words.unsettled}</span>
      </Text>
    </div>
  );
}

const taughtMessage = ({ outcome, word }: DictationWordChange) => {
  switch (outcome) {
    case "added":
      return t("dictation-island-word-added", { word });
    case "alreadyListed":
      return t("dictation-island-word-already-listed", { word });
    case "removed":
      return t("dictation-island-word-removed", { word });
    case "notListed":
      return t("dictation-island-word-not-listed", { word });
    case "failed":
      return t("dictation-island-word-failed", { word });
  }
};

/** What came of each word added or removed by voice. */
function TaughtWords({ taught }: { taught: DictationWordChange[] }) {
  return (
    <div aria-live="polite" className={cn("flex flex-col", WORDS_WIDTH)}>
      {taught.map((change, index) => (
        <Text
          as="p"
          className="px-inset-sm text-fg-secondary"
          key={`${String(index)}-${change.word}`}
          variant="caption"
        >
          {taughtMessage(change)}
        </Text>
      ))}
    </div>
  );
}

export function DictationIsland({
  isShown = true,
  level,
  notch,
  onAction,
  onHidden,
  onShapeChange,
  state,
}: DictationIslandProps) {
  // The words stay as they were once listening stops, while they go in.
  const [words, setWords] = useState<Words>({ settled: "", unsettled: "" });
  if (
    state.phase === "listening" &&
    (state.settled !== words.settled || state.unsettled !== words.unsettled)
  )
    setWords({ settled: state.settled, unsettled: state.unsettled });

  const title =
    state.phase === "listening" ? (
      <IslandRowTitle
        glyph={
          <span className="relative flex size-island-control">
            <LevelFill icon={Mic} level={level} />
          </span>
        }
        key="listening"
        title={t("dictation-island-listening")}
      />
    ) : state.phase === "finishing" ? (
      <IslandRowTitle
        glyph={
          <CircularProgress
            aria-label={t("dictation-island-typing")}
            isIndeterminate
            size="small"
          />
        }
        key="finishing"
        title={<span role="status">{t("dictation-island-typing")}</span>}
      />
    ) : (
      <IslandRowTitle
        glyph={<Mic aria-hidden />}
        key="problem"
        title={t("dictation-island")}
      />
    );

  const end =
    state.phase === "listening" ? (
      <span className="flex items-center gap-1" key="listening">
        <Close
          label={t("dictation-island-cancel")}
          onPress={() => {
            onAction("cancel");
          }}
        />
        <Button
          color="primary"
          onPress={() => {
            onAction("stop");
          }}
        >
          {t("dictation-island-done")}
        </Button>
      </span>
    ) : state.phase === "problem" || state.phase === "taught" ? (
      <span className="flex" key="problem">
        <Close
          label={t("dictation-island-close")}
          onPress={() => {
            onAction("close");
          }}
        />
      </span>
    ) : null;

  const taught =
    state.phase === "listening" || state.phase === "taught" ? state.taught : [];
  const section =
    state.phase === "problem" ? (
      <div className="w-80" key={`problem-${state.problem}`}>
        <Alert
          action={
            isFixedInSettings(state.problem) ? (
              <Button
                onPress={() => {
                  onAction("openSettings");
                }}
              >
                {t("dictation-island-open-settings")}
              </Button>
            ) : undefined
          }
          color="error"
          role="alert"
        >
          {problemMessage(state.problem)}
        </Alert>
      </div>
    ) : state.phase === "taught" ? (
      <div key="taught">
        <TaughtWords taught={taught} />
      </div>
    ) : state.phase === "listening" || words.settled || words.unsettled ? (
      <div className="flex flex-col gap-inline" key="words">
        <HeardWords
          atPauses={state.phase === "listening" && state.atPauses}
          words={words}
        />
        {taught.length > 0 ? <TaughtWords taught={taught} /> : null}
      </div>
    ) : null;

  return (
    <Island
      aria-label={t("dictation-island")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={
        <IslandRow
          end={<IslandParts direction="row">{end}</IslandParts>}
          start={<IslandParts direction="row">{title}</IslandParts>}
        />
      }
      section={section ? <IslandParts>{section}</IslandParts> : undefined}
    />
  );
}
