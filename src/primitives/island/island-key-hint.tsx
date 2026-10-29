// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { splitAtPlaceable } from "../../i18n/split-at-placeable";
import { hotkeyKeys } from "../../lib/hotkey";
import { cn } from "../../lib/styling";
import { Keyboard } from "../keyboard/keyboard";
import { Text } from "../text/text";

// A hint on the island's text column naming the key a gesture is held with,
// drawn as keycaps wherever the language puts the key: the message is
// formatted with a stand-in for the key and split around it.
export function IslandKeyHint({
  binding,
  className,
  message,
}: {
  /** The key as a hotkey binding names it, such as `Alt` or
   * `CommandOrControl`, drawn as the platform names it. */
  binding: string;
  /** The hint, given the key's placeable. */
  message: (key: string) => string;
  className?: string;
}) {
  const [before, after] = splitAtPlaceable(message);
  return (
    <Text
      as="p"
      className={cn("px-inset-sm text-fg-secondary", className)}
      variant="caption"
    >
      {before}
      {hotkeyKeys(binding).map((key) => (
        <Keyboard className="align-middle" key={key}>
          {key}
        </Keyboard>
      ))}
      {after}
    </Text>
  );
}
