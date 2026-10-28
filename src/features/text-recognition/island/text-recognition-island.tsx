// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AlertCircle,
  QrCode,
  ScanText,
  SquareDashedMousePointer,
  X,
} from "lucide-react";
import { Fragment, type ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";

import type { TextRecognitionIslandAction } from "../../../bindings/TextRecognitionIslandAction";
import type { TextRecognitionIslandState } from "../../../bindings/TextRecognitionIslandState";

// Text Recognition on the island. While an area is drawn it sits at the top
// of the screen and says what to do; once one is drawn it hangs by it, a
// spinner while it is read, then what can be done with what was found. Each
// phase is its own row, revealed as the black changes shape round it.

export type TextRecognitionIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
> & {
  onAction: (action: TextRecognitionIslandAction) => void;
  state: TextRecognitionIslandState;
  /** Which coming-out this is (`IslandHost`). */
  showing?: number;
};

/** What is going on, said as the island's row titles say what is in hand;
 * the words are announced as they change. */
function Status({
  children,
  glyph,
  isError = false,
}: {
  children: string;
  glyph: ReactNode;
  isError?: boolean;
}) {
  return (
    <IslandRowTitle
      glyph={isError ? <span className="flex text-error">{glyph}</span> : glyph}
      title={<span role="status">{children}</span>}
    />
  );
}

function Close({ onAction }: Pick<TextRecognitionIslandProps, "onAction">) {
  return (
    <IconButton
      aria-label={t("text-recognition-island-close")}
      className="text-fg-secondary enabled:hover:text-fg"
      onPress={() => {
        onAction("close");
      }}
    >
      <X aria-hidden className="size-icon" />
    </IconButton>
  );
}

/** Reading again from the top, for an area drawn in the wrong place. */
function SelectAnother({
  onAction,
}: Pick<TextRecognitionIslandProps, "onAction">) {
  return (
    <IconButton
      aria-label={t("text-recognition-island-select-another")}
      onPress={() => {
        onAction("selectAnotherArea");
      }}
    >
      <SquareDashedMousePointer aria-hidden className="size-icon" />
    </IconButton>
  );
}

function Found({
  onAction,
  state,
}: Pick<TextRecognitionIslandProps, "onAction"> & {
  state: Extract<TextRecognitionIslandState, { phase: "ready" }>;
}) {
  if (state.lines > 0) {
    return (
      <span className="flex items-center gap-1">
        <Button
          color="primary"
          onPress={() => {
            onAction("copy");
          }}
        >
          {state.selected
            ? t("text-recognition-island-copy")
            : t("text-recognition-island-copy-all")}
        </Button>
        <Button
          onPress={() => {
            onAction("copyParagraph");
          }}
        >
          {t("text-recognition-island-copy-paragraph")}
        </Button>
      </span>
    );
  }
  if (state.qrCodes > 0) {
    return (
      <Status glyph={<QrCode aria-hidden />}>
        {t("text-recognition-island-qr-hint")}
      </Status>
    );
  }
  return (
    <Status glyph={<ScanText aria-hidden />}>
      {t("text-recognition-island-nothing-found")}
    </Status>
  );
}

function row({ onAction, state }: TextRecognitionIslandProps) {
  switch (state.phase) {
    case "choosing":
      return (
        <IslandRow
          end={<Close onAction={onAction} />}
          start={
            state.failed ? (
              <Status glyph={<AlertCircle aria-hidden />} isError>
                {t("text-recognition-island-failed")}
              </Status>
            ) : (
              <Status glyph={<ScanText aria-hidden />}>
                {t("text-recognition-island-choosing")}
              </Status>
            )
          }
        />
      );
    case "reading":
      return (
        <IslandRow
          end={<Close onAction={onAction} />}
          start={
            <Status
              glyph={
                <CircularProgress
                  aria-label={t("text-recognition-island-reading-label")}
                  isIndeterminate
                  size="small"
                />
              }
            >
              {t("text-recognition-island-reading")}
            </Status>
          }
        />
      );
    case "ready":
      return (
        <IslandRow
          end={
            <span className="flex items-center gap-1">
              <SelectAnother onAction={onAction} />
              <Close onAction={onAction} />
            </span>
          }
          start={<Found onAction={onAction} state={state} />}
        />
      );
  }
}

export function TextRecognitionIsland({
  isShown = true,
  notch,
  onAction,
  onHidden,
  onShapeChange,
  showing = 0,
  state,
}: TextRecognitionIslandProps) {
  // Each phase's row is mounted afresh, so it is revealed where it sits as
  // the island changes shape round it rather than cut in.
  const shown =
    state.phase === "ready" ? `ready-${String(state.lines > 0)}` : state.phase;
  return (
    <Island
      aria-label={t("text-recognition-island")}
      isShown={isShown}
      // A new island each time it moves between the top of the screen and an
      // area, so it comes out where it now is.
      key={showing}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={<Fragment key={shown}>{row({ onAction, state })}</Fragment>}
    />
  );
}
