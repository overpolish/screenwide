// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ref } from "react";

import { Button } from "../../components/base/button/button";
import { Text } from "../../components/base/text/text";
import { WindowShell } from "../../components/shared/window-shell/window-shell";
import { t } from "../../i18n/i18n";

export type AlertProps = {
  message: string;
  onDismiss: () => void;
  title: string;
  /** Measured by the window so it can fit itself to the words. */
  contentRef?: Ref<HTMLElement>;
};

/**
 * A failure the user has to hear about, as it looks: the only place its
 * shell exists, so the real window and the stories cannot drift apart.
 *
 * Built as the confirmation sheet is, with one button: there is nothing to
 * decide, only something to read. Return and Escape both dismiss it, so the
 * button need not hold focus and draw a ring on arrival.
 */
export function Alert({ contentRef, message, onDismiss, title }: AlertProps) {
  return (
    <WindowShell height="content" ref={contentRef}>
      <div className="flex flex-col gap-section p-window-inset" role="alert">
        <Text variant="headline">{title}</Text>
        <Text className="select-text">{message}</Text>
        <div className="flex justify-end">
          <Button color="primary" onPress={onDismiss}>
            {t("alert-dismiss")}
          </Button>
        </div>
      </div>
    </WindowShell>
  );
}
