// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AlertCircle,
  QrCode,
  ScanText,
  SquareDashedMousePointer,
  X,
} from "lucide-react";
import { AnimatePresence } from "motion/react";
import { Fragment, type ReactNode, useState } from "react";

import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";

import { type QrCodeFailure, QrCodeRow, QrCodeSection } from "./qr-code-island";

import type { TextRecognitionIslandAction } from "../../../bindings/TextRecognitionIslandAction";
import type { TextRecognitionIslandState } from "../../../bindings/TextRecognitionIslandState";

// Text Recognition on the island. While an area is drawn it sits at the top
// of the screen and says what to do; once one is drawn it hangs by it, a
// spinner while it is read, then what can be done with what was found, or
// what a QR code pressed in it holds. Each phase is its own row, revealed as
// the black changes shape round it.

export type TextRecognitionIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
> & {
  onAction: (action: TextRecognitionIslandAction) => void;
  /** Copies what a QR code holds; rejects when it could not. */
  onCopyQrCode: (content: string) => Promise<unknown>;
  /** Opens a QR code's action; rejects when it could not. */
  onOpenQrCode: (url: string) => Promise<unknown>;
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
      <>
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
      </>
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

function row({
  onAction,
  state,
}: Pick<TextRecognitionIslandProps, "onAction"> & {
  state: Exclude<TextRecognitionIslandState, { phase: "qr" }>;
}) {
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
            <>
              <SelectAnother onAction={onAction} />
              <Close onAction={onAction} />
            </>
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
  onCopyQrCode,
  onHidden,
  onOpenQrCode,
  onShapeChange,
  showing = 0,
  state,
}: TextRecognitionIslandProps) {
  // What failed with the QR code on the island, forgotten with the code.
  const [qrFailure, setQrFailure] = useState<{
    content: string;
    failure: QrCodeFailure | null;
  } | null>(null);
  const qr = state.phase === "qr" ? state : null;
  const failure =
    qr && qrFailure?.content === qr.content ? qrFailure.failure : null;
  // Each phase's row is mounted afresh, so it is revealed where it sits as
  // the island changes shape round it rather than cut in.
  const shown =
    state.phase === "ready" ? `ready-${String(state.lines > 0)}` : state.phase;
  return (
    <Island
      // Hung from the end and spanning the row, what a QR code holds is held
      // to the row's width as the black glides (`IslandSection`).
      anchor={qr ? "end" : "center"}
      aria-label={t("text-recognition-island")}
      isShown={isShown}
      // A new island each time it moves between the top of the screen and an
      // area, so it comes out where it now is.
      key={showing}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={
        <Fragment key={shown}>
          {state.phase === "qr" ? (
            <QrCodeRow
              content={state.content}
              decodeError={state.decodeError}
              onAction={onAction}
              onCopy={onCopyQrCode}
              onFailure={(next) => {
                setQrFailure({ content: state.content, failure: next });
              }}
              onOpen={onOpenQrCode}
            />
          ) : (
            row({ onAction, state })
          )}
        </Fragment>
      }
      section={
        qr ? (
          // Another code pressed swaps what it holds in, as the row swaps.
          <AnimatePresence initial={false} mode="popLayout">
            <IslandCrossfade key={qr.content}>
              <QrCodeSection
                content={qr.content}
                decodeError={qr.decodeError}
                failure={failure}
              />
            </IslandCrossfade>
          </AnimatePresence>
        ) : undefined
      }
      sectionSpansRow={qr !== null}
    />
  );
}
