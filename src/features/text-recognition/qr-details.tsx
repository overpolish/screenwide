// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { Text } from "../../components/base/text/text";
import { CopyableText } from "../../components/shared/copyable-text/copyable-text";
import { WindowHeader } from "../../components/shared/window-header/window-header";
import { WindowShell } from "../../components/shared/window-shell/window-shell";
import { t } from "../../i18n/i18n";

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
      <div className="gap-section px-window-inset pb-window-inset flex min-h-0 grow flex-col">
        <Text>{description(payload)}</Text>
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
          <footer className="flex items-center justify-end">
            <Button color="primary" onPress={onAction}>
              {payload.label}
            </Button>
          </footer>
        ) : null}
      </div>
    </WindowShell>
  );
}
