// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { NotebookPen, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useFocusWithin } from "react-aria";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { Island } from "../../../primitives/island/island";
import { IslandParts } from "../../../primitives/island/island-parts";
import { type IslandProps } from "../../../primitives/island/island-props";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";
import { clock } from "../../recording-controls/island/island-names";

import { notesSection } from "./notes-section";

import type { NotesIslandAction } from "../../../bindings/NotesIslandAction";
import type { NotesIslandState } from "../../../bindings/NotesIslandState";

// Meeting notes on the island, at the top of the screen. While they are
// taken the island keeps out of the way: the notebook glyph alone, and under
// the pointer or focus how long they have run, Discard and Done beside it.
// Stopped, a spinner alone says the speakers are being told apart; saved,
// the row offers to show the notes. A problem says why under the row, with
// Settings where it is put right.

/** The notes as the island shows them. */
export type NotesParts = {
  onAction: (action: NotesIslandAction) => void;
  state: NotesIslandState;
  // The clock's own when not given, as stories fix it.
  /** The time now, in ms since the Unix epoch. */
  now?: number;
};

export type NotesIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
> &
  NotesParts;

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

/** The notes' row, on their own island or on the recording island, which
 * reshapes into it from the controls as it does into a recording. */
export function NotesRow({
  isHovered,
  now: fixedNow,
  onAction,
  state,
}: NotesParts & { isHovered: boolean }) {
  const ticking = useNow(state.phase === "taking" && fixedNow === undefined);
  const now = fixedNow ?? ticking;
  const [hasFocus, setHasFocus] = useState(false);
  const { focusWithinProps } = useFocusWithin({
    onFocusWithinChange: setHasFocus,
  });

  if (state.phase === "taking" || state.phase === "finishing") {
    const elapsed = clock(
      state.phase === "taking" ? Math.max(0, now - state.startedMs) / 1_000 : 0,
    );
    const showsControls = state.phase === "taking" && (isHovered || hasFocus);
    return (
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
  }
  return (
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

export function NotesIsland({
  isShown = true,
  notch,
  onHidden,
  onShapeChange,
  ...notes
}: NotesIslandProps) {
  const [isHovered, setIsHovered] = useState(false);
  return (
    <Island
      aria-label={t("notes-island")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onHoverChange={setIsHovered}
      onShapeChange={onShapeChange}
      row={<NotesRow {...notes} isHovered={isHovered} />}
      section={notesSection(notes)}
    />
  );
}
