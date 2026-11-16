// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { NotebookPen, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useFocusWithin } from "react-aria";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";
import { Alert } from "../../../primitives/alert/alert";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { Island } from "../../../primitives/island/island";
import { IslandParts } from "../../../primitives/island/island-parts";
import { type IslandProps } from "../../../primitives/island/island-props";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";
import { clock } from "../../recording-controls/island/island-names";

import type { NotesIslandAction } from "../../../bindings/NotesIslandAction";
import type { NotesIslandState } from "../../../bindings/NotesIslandState";
import type { NotesProblem } from "../../../bindings/NotesProblem";

// Meeting notes on the island, at the top of the screen. While they are
// taken the island keeps out of the way: the notebook glyph alone, and under
// the pointer or focus how long they have run, Discard and Done beside it.
// Stopped, a spinner alone says the speakers are being told apart; saved,
// the row offers to show the notes. A problem says why under the row, with
// Settings where it is put right.

export type NotesIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
> & {
  onAction: (action: NotesIslandAction) => void;
  state: NotesIslandState;
  /** The time now, in milliseconds since the Unix epoch; the clock's own
   * when not given, as stories fix it. */
  now?: number;
};

const problemMessage = (problem: NotesProblem) => {
  switch (problem) {
    case "noModel":
      return t("notes-island-no-model");
    case "noMicrophone":
      return t("notes-island-no-microphone");
    case "noSystemAudio":
      return t("notes-island-no-system-audio");
    case "failed":
      return t("notes-island-failed");
  }
};

/** Whether Settings is where `problem` is put right. */
const isFixedInSettings = (problem: NotesProblem) => problem === "noModel";

/** The time now, a second at a time, while `ticks`. */
function useNow(ticks: boolean) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!ticks) return;
    const timer = window.setInterval(() => {
      setNow(Date.now());
    }, 1_000);
    return () => {
      window.clearInterval(timer);
    };
  }, [ticks]);
  return now;
}

function Close({ onPress }: { onPress: () => void }) {
  return (
    <IconButton
      aria-label={t("notes-island-close")}
      className="text-fg-secondary enabled:hover:text-fg"
      onPress={onPress}
    >
      <X aria-hidden className="size-icon" />
    </IconButton>
  );
}

export function NotesIsland({
  isShown = true,
  notch,
  now: fixedNow,
  onAction,
  onHidden,
  onShapeChange,
  state,
}: NotesIslandProps) {
  const ticking = useNow(state.phase === "taking" && fixedNow === undefined);
  const now = fixedNow ?? ticking;
  const [isHovered, setIsHovered] = useState(false);
  const [hasFocus, setHasFocus] = useState(false);
  const { focusWithinProps } = useFocusWithin({
    onFocusWithinChange: setHasFocus,
  });

  let row;
  if (state.phase === "taking" || state.phase === "finishing") {
    const elapsed = clock(
      state.phase === "taking" ? Math.max(0, now - state.startedMs) / 1_000 : 0,
    );
    const showsControls = state.phase === "taking" && (isHovered || hasFocus);
    row = (
      <IslandRow
        end={
          <span className="contents" {...focusWithinProps}>
            <IslandParts
              className="flex items-center gap-separate"
              direction="row"
            >
              {state.phase === "finishing" ? (
                <span
                  className="flex size-island-control items-center justify-center"
                  key="finishing"
                  role="status"
                >
                  <CircularProgress
                    aria-label={t("notes-island-finishing")}
                    isIndeterminate
                    size="small"
                  />
                </span>
              ) : (
                // Focusable, so the keyboard brings out the controls as the
                // pointer does.
                <span
                  aria-label={t("notes-island-taking")}
                  className="flex size-island-control items-center justify-center rounded-full ring-focus outline-none focus-visible:ring-3"
                  key="glyph"
                  role="img"
                  tabIndex={0}
                >
                  <NotebookPen aria-hidden className="size-icon" />
                </span>
              )}
              {showsControls ? (
                <span className="flex items-center gap-1" key="controls">
                  <span
                    aria-label={t("notes-island-elapsed", { time: elapsed })}
                    className="px-2 text-label tabular-nums"
                    role="timer"
                  >
                    {elapsed}
                  </span>
                  <ConfirmActionButton
                    confirmLabel={t("notes-island-confirm-discard")}
                    icon={<X />}
                    label={t("notes-island-discard")}
                    onConfirm={() => {
                      onAction("discard");
                    }}
                    variant="ghost"
                  />
                  <Button
                    color="primary"
                    onPress={() => {
                      onAction("stop");
                    }}
                  >
                    {t("notes-island-done")}
                  </Button>
                </span>
              ) : null}
            </IslandParts>
          </span>
        }
      />
    );
  } else {
    row = (
      <IslandRow
        end={
          <IslandParts direction="row">
            {state.phase === "saved" ? (
              <span className="flex items-center gap-1" key="saved">
                <Close
                  onPress={() => {
                    onAction("close");
                  }}
                />
                <Button
                  onPress={() => {
                    onAction("open");
                  }}
                >
                  {t("notes-island-open")}
                </Button>
              </span>
            ) : (
              <span className="flex" key="problem">
                <Close
                  onPress={() => {
                    onAction("close");
                  }}
                />
              </span>
            )}
          </IslandParts>
        }
        start={
          <IslandRowTitle
            glyph={<NotebookPen aria-hidden />}
            title={
              state.phase === "saved"
                ? t("notes-island-saved")
                : t("notes-island")
            }
          />
        }
      />
    );
  }

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
                {t("notes-island-open-settings")}
              </Button>
            ) : undefined
          }
          color="error"
          role="alert"
        >
          {problemMessage(state.problem)}
        </Alert>
      </div>
    ) : null;

  return (
    <Island
      aria-label={t("notes-island")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onHoverChange={setIsHovered}
      onShapeChange={onShapeChange}
      row={row}
      section={section ? <IslandParts>{section}</IslandParts> : undefined}
    />
  );
}
