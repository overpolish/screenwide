// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ExternalLink } from "lucide-react";

import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { Text } from "../../components/base/text/text";
import { CopyableText } from "../../components/shared/copyable-text/copyable-text";
import { WindowHeader } from "../../components/shared/window-header/window-header";

import type { QrPayload } from "./qr-code-payload";

export type QrDetailsProps = {
  content: string;
  onClose: () => void;
  onCopy: () => unknown;
  payload: QrPayload;
  error?: string;
  onAction?: () => void;
};

const description = (payload: QrPayload) => {
  if (payload.kind === "action")
    return "This QR code contains an action you can open or copy.";
  if (payload.kind === "unsupported")
    return "Screenwide cannot perform this QR code’s action.";
  return "This QR code contains information you can copy.";
};

export function QrDetails({
  content,
  error,
  onAction,
  onClose,
  onCopy,
  payload,
}: QrDetailsProps) {
  const status =
    error ?? (payload.kind === "unsupported" ? payload.reason : undefined);

  return (
    <main className="window-surface gap-section flex h-full w-full flex-col overflow-hidden rounded-window text-content-fg">
      <WindowHeader onClose={onClose} title={payload.label} />

      <div className="gap-section px-window-inset pb-window-inset flex min-h-0 grow flex-col">
        <Text>{description(payload)}</Text>
        {status ? (
          <Alert color="error" role="alert">
            {status}
          </Alert>
        ) : null}

        <CopyableText
          className="grow"
          label="Detected content"
          onCopy={onCopy}
          value={content}
        />

        {payload.kind === "action" && onAction ? (
          <footer className="flex items-center justify-end">
            <Button color="primary" onPress={onAction}>
              <ExternalLink aria-hidden />
              {payload.label}
            </Button>
          </footer>
        ) : null}
      </div>
    </main>
  );
}
