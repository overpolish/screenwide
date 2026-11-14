// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useDateFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { Button } from "../../../../primitives/button/button";

import type { OpenverseFailure } from "./openverse-api";

// Why a search online found nothing to show. Too many searches this minute,
// or no answer, can be tried again at once; a day's searches used up come
// back only when the day ends, which it says in the asker's own time, so it
// offers nothing to press.

export function OpenverseFailureAlert({
  failure,
  onRetry,
}: {
  failure: OpenverseFailure;
  /** Asks again for the page that failed. */
  onRetry: () => void;
}) {
  const time = useDateFormatter({ timeStyle: "short" });
  if (typeof failure === "object") {
    const until = new Date(failure.usedUp.until);
    return (
      <Alert color="warning">
        {t("editor-island-openverse-used-up", {
          midnight:
            until.getHours() === 0 && until.getMinutes() === 0 ? "yes" : "no",
          time: time.format(until),
        })}
      </Alert>
    );
  }
  // With its action it keeps to the start, so its icon sits on the text
  // column, under the heading, and the action at the line's end.
  return (
    <Alert
      action={
        <Button onPress={onRetry}>{t("editor-island-openverse-retry")}</Button>
      }
      className="ps-inset-sm"
      color="error"
    >
      {failure === "busy"
        ? t("editor-island-openverse-busy")
        : t("editor-island-openverse-unavailable")}
    </Alert>
  );
}
