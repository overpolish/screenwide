// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowLeft, QrCode, X } from "lucide-react";
import { useState } from "react";

import { CopyButton } from "../../../composites/copy-button/copy-button";
import { t } from "../../../i18n/i18n";
import { Alert } from "../../../primitives/alert/alert";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { IslandParts } from "../../../primitives/island/island-parts";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { Text } from "../../../primitives/text/text";
import { classifyQrPayload, type QrPayload } from "../qr-code-payload";

import type { TextRecognitionIslandAction } from "../../../bindings/TextRecognitionIslandAction";

// A QR code pressed in what was found, on the island in place of what was
// found: what the code is at the row's start, with Back to what was found
// before it; what can be done with it at the row's end, its action first
// where it has one; and under the row what it holds, as wide as the row, to
// read and select, cut to a few lines with Show all where it runs long. What
// cannot be done, and what failed, is said as an alert under it. Another
// code pressed swaps what changes with it (`IslandParts`), as the black
// glides to the new row's size.

/** Past this many characters what a code holds is cut to a few lines. */
const LONG_CONTENT = 140;

export type QrCodeFailure = "copy" | "open";

export type QrCodeHandlers = {
  onAction: (action: TextRecognitionIslandAction) => void;
  /** Copies what the code holds; rejects when it could not. */
  onCopy: (content: string) => Promise<unknown>;
  /** Opens the code's action; rejects when it could not. */
  onOpen: (url: string) => Promise<unknown>;
};

/** What the code is, as the row's title says it. */
const title = (payload: QrPayload) => {
  if (payload.kind !== "action") return payload.label;
  switch (payload.action) {
    case "email":
      return t("text-recognition-qr-email-address");
    case "phone":
      return t("text-recognition-qr-phone");
    case "sms":
      return t("text-recognition-qr-sms");
    case "link":
      return t("text-recognition-qr-link");
  }
};

const quiet = "text-fg-secondary enabled:hover:text-fg";

export function QrCodeRow({
  content,
  decodeError,
  onAction,
  onCopy,
  onFailure,
  onOpen,
}: QrCodeHandlers & {
  content: string;
  decodeError: string | null;
  onFailure: (failure: QrCodeFailure | null) => void;
}) {
  const payload = classifyQrPayload(content, decodeError ?? undefined);
  // Back keeps its place at the row's start while the rest gives way to the
  // next code's, as the island's parts do (`IslandParts`).
  return (
    <IslandRow
      end={
        <IslandParts direction="row">
          <div className="flex items-center gap-separate" key={content}>
            {payload.kind === "action" ? (
              <Button
                color="primary"
                onPress={() => {
                  onFailure(null);
                  onOpen(payload.url).catch(() => {
                    onFailure("open");
                  });
                }}
              >
                {payload.label}
              </Button>
            ) : null}
            {content ? (
              <CopyButton
                aria-label={t("text-recognition-island-qr-copy")}
                onCopy={() => {
                  onFailure(null);
                  return onCopy(content).catch((cause: unknown) => {
                    onFailure("copy");
                    throw cause;
                  });
                }}
              />
            ) : null}
            <IconButton
              aria-label={t("text-recognition-island-close")}
              className={quiet}
              onPress={() => {
                onAction("close");
              }}
            >
              <X aria-hidden className="size-icon" />
            </IconButton>
          </div>
        </IslandParts>
      }
      start={
        <>
          <IconButton
            aria-label={t("text-recognition-island-qr-back")}
            className={quiet}
            onPress={() => {
              onAction("closeQrCode");
            }}
          >
            <ArrowLeft aria-hidden className="size-icon rtl:-scale-x-100" />
          </IconButton>
          <IslandParts direction="row">
            <IslandRowTitle
              glyph={<QrCode aria-hidden />}
              key={content}
              title={<span role="status">{title(payload)}</span>}
            />
          </IslandParts>
        </>
      }
    />
  );
}

/** What the code holds, selectable in its own direction, and what went
 * wrong with it. */
export function QrCodeSection({
  content,
  decodeError,
  failure,
}: {
  content: string;
  decodeError: string | null;
  failure: QrCodeFailure | null;
}) {
  const [isExpanded, setIsExpanded] = useState(false);
  const payload = classifyQrPayload(content, decodeError ?? undefined);
  const isLong =
    content.length > LONG_CONTENT || content.split("\n").length > 3;
  // A link or code has no spaces to break at: `wrap-anywhere` breaks it
  // within the row and, unlike `break-words`, also lowers the width the
  // island measures it at, so a long one can never push into the padding.
  const text = (
    <Text
      as="p"
      className="whitespace-pre-wrap wrap-anywhere select-text"
      dir="auto"
      variant="body"
    >
      {content}
    </Text>
  );
  const problem =
    failure === "open" && payload.kind === "action"
      ? t("text-recognition-qr-action-failed", { action: payload.action })
      : failure === "copy"
        ? t("text-recognition-qr-copy-failed")
        : payload.kind === "unsupported"
          ? payload.reason
          : null;
  // As wide as the row and no wider (`w-0 min-w-full`, with the section
  // spanning the row): what the code holds wraps within it rather than
  // widening the island.
  return (
    <IslandParts className="flex w-0 min-w-full flex-col gap-inline">
      {content ? (
        <section
          aria-label={t("text-recognition-island-qr-content")}
          className="flex flex-col items-start gap-inline"
          key="content"
        >
          {isExpanded ? (
            <ScrollArea
              className="px-inset-sm"
              constrainHeight
              rootClassName="max-h-40 w-full"
            >
              {text}
            </ScrollArea>
          ) : (
            <div className="line-clamp-3 px-inset-sm">{text}</div>
          )}
          {isLong ? (
            <Button
              aria-expanded={isExpanded}
              onPress={() => {
                setIsExpanded((current) => !current);
              }}
              variant="ghost"
            >
              {isExpanded
                ? t("text-recognition-island-qr-show-less")
                : t("text-recognition-island-qr-show-all")}
            </Button>
          ) : null}
        </section>
      ) : null}
      {problem ? (
        <Alert
          className="px-inset-sm"
          color="error"
          key={`problem-${problem}`}
          role="alert"
        >
          {problem}
        </Alert>
      ) : null}
    </IslandParts>
  );
}
