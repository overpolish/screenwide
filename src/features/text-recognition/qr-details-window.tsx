// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useMemo, useState } from "react";

import {
  cancelTextRecognition,
  closeQrDetails,
  copyRecognitionContent,
  getQrDetails,
  type RecognizedQrCode,
} from "./api";
import { classifyQrPayload } from "./qr-code-payload";
import { QrDetails } from "./qr-details";

export function QrDetailsWindow() {
  const [code, setCode] = useState<RecognizedQrCode>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    let disposed = false;
    void getQrDetails()
      .then((current) => {
        if (!disposed) setCode(current);
      })
      .catch(() => undefined);
    const unlisten = listen<RecognizedQrCode>(
      "qr-details-updated",
      ({ payload }) => {
        setCode(payload);
        setError(undefined);
      },
    );
    return () => {
      disposed = true;
      void unlisten.then((cleanup) => {
        cleanup();
      });
    };
  }, []);

  const payload = useMemo(
    () => (code ? classifyQrPayload(code.content, code.decodeError) : undefined),
    [code],
  );
  const close = useCallback(() => {
    setError(undefined);
    void closeQrDetails();
  }, []);

  if (!code || !payload) return null;

  return (
    <QrDetails
      content={code.content}
      error={error}
      onAction={
        payload.kind === "action"
          ? () => {
              setError(undefined);
              void openUrl(payload.url).then(
                () => {
                  void cancelTextRecognition();
                },
                () => {
                  setError(`Could not ${payload.label.toLowerCase()}.`);
                },
              );
            }
          : undefined
      }
      onClose={close}
      onCopy={() => {
        setError(undefined);
        return copyRecognitionContent(code.content).catch((copyError: unknown) => {
          setError("Could not copy QR content.");
          throw copyError;
        });
      }}
      payload={payload}
    />
  );
}
