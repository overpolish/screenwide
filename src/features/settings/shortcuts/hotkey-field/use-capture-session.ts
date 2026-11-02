// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { t } from "../../../../i18n/i18n";

/** While a field records, the app's own shortcuts are suspended through
 * `onCaptureChange`, so the keys pressed reach the field rather than firing
 * what they already do; `onReady` says when that has happened. The window
 * losing focus ends the recording. Whatever happens, the shortcuts come
 * back when it ends. */
export function useCaptureSession({
  capturing,
  onCancel,
  onCaptureChange,
  onFailed,
  onReady,
}: {
  capturing: boolean;
  onCancel: () => void;
  onFailed: (message: string) => void;
  onReady: () => void;
  onCaptureChange?: (capturing: boolean) => void | Promise<void>;
}) {
  useEffect(() => {
    if (!capturing) return;
    let disposed = false;
    const started = Promise.resolve().then(() => onCaptureChange?.(true));
    void started
      .then(() => {
        if (!disposed) onReady();
      })
      .catch((reason: unknown) => {
        if (!disposed)
          onFailed(t("hotkey-field-capture-failed", { error: String(reason) }));
      });
    window.addEventListener("blur", onCancel);
    return () => {
      disposed = true;
      window.removeEventListener("blur", onCancel);
      // Always release, even when cancellation races asynchronous preparation.
      void started
        .catch(() => undefined)
        .then(() => onCaptureChange?.(false))
        .catch((reason: unknown) => {
          console.error("Could not restore shortcuts after capture", reason);
        });
    };
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- the callbacks only set the field's state and are remade each render; a new one must not end the recording and start it again.
  }, [capturing, onCaptureChange]);
}
