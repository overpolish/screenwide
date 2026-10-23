// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { splitAtPlaceable } from "../../../../i18n/split-at-placeable";
import { hotkeyKeys } from "../../../../lib/hotkey";
import { Keyboard } from "../../../../primitives/keyboard/keyboard";

/** The help line under the transcript, with the modifier a click takes drawn
 * as a key: Command on macOS, Ctrl on Windows. */
export function TranscriptHint() {
  const [before, after] = splitAtPlaceable((key) =>
    t("editor-panels-transcript-hint", { key }),
  );
  return (
    <p className="text-caption text-fg-secondary">
      {before}
      {hotkeyKeys("CommandOrControl").map((key) => (
        <Keyboard className="align-middle" key={key}>
          {key}
        </Keyboard>
      ))}
      {after}
    </p>
  );
}
