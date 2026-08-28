// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { createPortal } from "react-dom";

import { Button } from "../../components/base/button/button";
import { cn } from "../../lib/styling";

import { copyRecognitionContent } from "./api";
import { classifyQrPayload } from "./qr-code-payload";

import type { RecognizedQrCode } from "./api";
import type { QrPayload } from "./qr-code-payload";

type ActiveQr = {
  code: RecognizedQrCode;
  payload: QrPayload;
  error?: string;
};

const actionLabel = (payload: QrPayload, content: string) =>
  payload.kind === "action"
    ? `${payload.label}: ${content}`
    : `${payload.label}: show QR content`;

export function QrCodeOverlay({
  codes,
  onDismiss,
}: {
  codes: readonly RecognizedQrCode[];
  onDismiss: () => void;
}) {
  const [active, setActive] = useState<ActiveQr>();

  const copyAndDismiss = (content: string) => {
    void copyRecognitionContent(content).then(onDismiss, () => {
      setActive((current) =>
        current ? { ...current, error: "Could not copy QR content." } : current,
      );
    });
  };

  return (
    <>
      <div className="pointer-events-none absolute inset-0 z-10">
        {codes.map((code, index) => {
          const payload = classifyQrPayload(code.content, code.decodeError);
          const unsupported = payload.kind === "unsupported";
          return (
            <button
              aria-label={actionLabel(payload, code.content)}
              className={cn(
                "pointer-events-auto absolute grid cursor-pointer place-items-center rounded-[2px] outline outline-1 transition-colors",
                unsupported
                  ? "bg-error/20 outline-error/80 hover:bg-error/30 hover:outline-error"
                  : "bg-info/20 outline-info/65 hover:bg-info/35 hover:outline-info",
              )}
              key={`${code.content}-${index.toString()}`}
              onClick={() => {
                if (payload.kind !== "action") {
                  setActive({ code, payload });
                  return;
                }
                void openUrl(payload.url).then(onDismiss, () => {
                  setActive({
                    code,
                    error: `Could not ${payload.label.toLowerCase()}.`,
                    payload,
                  });
                });
              }}
              onPointerDown={(event) => {
                event.stopPropagation();
              }}
              style={{
                height: `${(code.bounds.height * 100).toString()}%`,
                left: `${(code.bounds.x * 100).toString()}%`,
                top: `${(code.bounds.y * 100).toString()}%`,
                width: `${(code.bounds.width * 100).toString()}%`,
              }}
              title={payload.label}
              type="button"
            >
              {unsupported && (
                <span className="rounded bg-error px-1.5 py-0.5 text-xs font-semibold whitespace-nowrap text-white shadow-sm">
                  Unsupported QR
                </span>
              )}
            </button>
          );
        })}
      </div>
      {active &&
        createPortal(
          <div
            aria-label="QR code content"
            className="pointer-events-auto fixed top-1/2 left-1/2 z-50 flex min-h-48 w-[min(28rem,calc(100vw-16px))] min-w-72 max-h-[calc(100vh-16px)] -translate-x-1/2 -translate-y-1/2 flex-col overflow-auto rounded-md border border-muted/20 bg-content/95 p-4 text-content-fg shadow-lg backdrop-blur-md"
            onPointerDown={(event) => {
              event.stopPropagation();
            }}
            role="dialog"
          >
            <div className="text-sm font-semibold">{active.payload.label}</div>
            {active.payload.kind === "unsupported" && (
              <div className="mt-1 text-xs text-error">
                {active.payload.reason}
              </div>
            )}
            {active.error && (
              <div className="mt-1 text-xs text-error">{active.error}</div>
            )}
            <pre className="mt-3 min-h-20 flex-1 overflow-auto rounded bg-neutral p-3 text-xs break-all whitespace-pre-wrap select-text">
              {active.code.content || "(empty)"}
            </pre>
            <div className="mt-2 flex justify-end gap-2">
              <Button
                onPress={() => {
                  setActive(undefined);
                }}
                size="compact"
                variant="ghost"
              >
                Close
              </Button>
              {active.code.content && (
                <Button
                  onPress={() => {
                    copyAndDismiss(active.code.content);
                  }}
                  size="compact"
                >
                  Copy content
                </Button>
              )}
            </div>
          </div>,
          document.body,
        )}
    </>
  );
}
