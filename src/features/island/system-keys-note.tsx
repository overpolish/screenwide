// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../i18n/i18n";
import { splitAtPlaceable } from "../../i18n/split-at-placeable";
import { hotkeyKeys } from "../../lib/hotkey";
import { Keyboard, Shortcut } from "../../primitives/keyboard/keyboard";
import { Text } from "../../primitives/text/text";

/** A hint under the island's row, said once: the system's keys just pressed
 * now open Screenwide, and where to give them back. It wraps at a reading
 * width rather than widening the island to one line. */
export function SystemKeysNote({ keys }: { keys: string }) {
  const [before, after] = splitAtPlaceable((mark) =>
    t("settings-system-keys-announce", { keys: mark }),
  );
  return (
    <Text
      as="p"
      className="w-max max-w-80 px-inset text-fg-secondary"
      role="status"
      variant="caption"
    >
      {before}
      <Shortcut className="align-middle">
        {hotkeyKeys(keys).map((key) => (
          <Keyboard key={key}>{key}</Keyboard>
        ))}
      </Shortcut>
      {after}
    </Text>
  );
}
