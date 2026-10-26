// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { WindowHeader } from "../../composites/window-header/window-header";
import { WindowShell } from "../../composites/window-shell/window-shell";
import { t } from "../../i18n/i18n";
import { Alert } from "../../primitives/alert/alert";
import { Button } from "../../primitives/button/button";
import { Text } from "../../primitives/text/text";

import { CopyableText } from "./copyable-text";

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
    return t("text-recognition-qr-action-description");
  if (payload.kind === "unsupported")
    return t("text-recognition-qr-unsupported-description");
  return t("text-recognition-qr-information-description");
};

// What a QR code found on screen holds, titled by what it is: a line on what
// can be done with it, why not when it cannot be used, the content to read
// and copy, and the action it offers as the window's one primary button.
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
    <WindowShell
      header={<WindowHeader onClose={onClose} title={payload.label} />}
    >
      <div className="flex min-h-0 grow flex-col gap-separate px-inset pb-inset">
        <Text className="text-fg-secondary">{description(payload)}</Text>
        {status ? (
          <Alert color="error" role="alert">
            {status}
          </Alert>
        ) : null}

        <CopyableText
          className="grow"
          copyLabel={t("text-recognition-qr-copy-content")}
          label={t("text-recognition-qr-content")}
          onCopy={onCopy}
          value={content}
        />

        {payload.kind === "action" && onAction ? (
          <footer className="flex items-center justify-end gap-separate">
            <Button color="primary" onPress={onAction}>
              {payload.label}
            </Button>
          </footer>
        ) : null}
      </div>
    </WindowShell>
  );
}
