// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { splitAtPlaceable } from "../../../../i18n/split-at-placeable";
import { hotkeyKeys } from "../../../../lib/hotkey";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { Keyboard } from "../../../../primitives/keyboard/keyboard";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";
import { Text } from "../../../../primitives/text/text";
import { EDITOR_ISLAND_WIDTH } from "../editor-island-width";

import type { LullIslandProps } from "./lull-island";

// A lull's choices, on the line under its row: how it plays, and its trim.
// How it plays is a choice, so it is a list with a tick; a trim is done at
// once, so it is a one-shot action at the end of its own row. Beneath them
// hangs the key that turns dragging its end from a retime into a trim, a
// part only as wide as it says.

export function LullChoices({
  onPlayChange,
  onTrimToBreath,
  play,
}: LullIslandProps) {
  const speed = useNumberFormatter({
    maximumFractionDigits: 1,
    minimumFractionDigits: 1,
  });
  const retimed =
    play.kind !== "retimed"
      ? null
      : play.speed > 1
        ? t("editor-island-lull-retimed-faster", {
            speed: speed.format(play.speed),
          })
        : t("editor-island-lull-retimed-slower", {
            speed: speed.format(play.speed),
          });
  return (
    // Inset as the row's content is, with room beneath before the hint; at
    // the width the editor island's parts share, so the hint hangs as a
    // part of its own rather than a sliver short of the island's width.
    <div
      className={cn(
        "flex flex-col gap-separate px-1 pb-inset-sm",
        EDITOR_ISLAND_WIDTH.line,
      )}
    >
      <ListBox
        aria-label={t("editor-island-lull-plays")}
        disallowEmptySelection
        onSelectionChange={(keys) => {
          if (keys === "all") return;
          const [choice] = keys;
          if (choice === "keep" || choice === "squeeze") onPlayChange(choice);
        }}
        selectedKeys={[play.kind]}
        selectionMode="single"
      >
        <ListBoxItem id="squeeze">
          {t("editor-island-lull-squeeze")}
        </ListBoxItem>
        <ListBoxItem id="keep">{t("editor-island-lull-keep")}</ListBoxItem>
        {retimed === null ? null : (
          <ListBoxItem id="retimed">{retimed}</ListBoxItem>
        )}
      </ListBox>
      <div className="flex justify-end">
        <Button onPress={onTrimToBreath}>
          {t("editor-island-lull-trim-to-breath")}
        </Button>
      </div>
    </div>
  );
}

/** The key that, held while dragging the lull's end, trims it rather than
 * retiming it, drawn as a keycap wherever the language puts it: ⌥ on a
 * Mac, Alt on Windows. Room at its sides keeps the small part it hangs as
 * from sitting tight round the words. */
export function LullHint() {
  const [before, after] = splitAtPlaceable((key) =>
    t("editor-island-lull-hint", { key }),
  );
  return (
    <Text as="p" className="px-inset-sm text-fg-secondary" variant="caption">
      {before}
      <Keyboard className="align-middle">{hotkeyKeys("AltLeft")[0]}</Keyboard>
      {after}
    </Text>
  );
}
