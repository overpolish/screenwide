// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { splitAtPlaceable } from "../../../../i18n/split-at-placeable";
import { hotkeyKeys } from "../../../../lib/hotkey";
import { Button } from "../../../../primitives/button/button";
import { Keyboard } from "../../../../primitives/keyboard/keyboard";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";
import { Text } from "../../../../primitives/text/text";

import type { LullIslandProps } from "./lull-island";

// What hangs under a lull's row: how it plays, its trim, and the key that
// turns dragging its end from a retime into a trim. How it plays is a
// choice, so it is a list with a tick; a trim is done at once, so it is a
// button.

export function LullSection({
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
    <div className="flex w-56 flex-col gap-separate">
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
      <Button className="w-full" onPress={onTrimToBreath}>
        {t("editor-island-lull-trim-to-breath")}
      </Button>
      <Text as="p" className="px-inset-sm text-fg-secondary" variant="caption">
        <TrimHint />
      </Text>
    </div>
  );
}

/** The key that, held while dragging the lull's end, trims it rather than
 * retiming it, drawn as a keycap wherever the language puts it: ⌥ on a
 * Mac, Alt on Windows. */
function TrimHint() {
  const [before, after] = splitAtPlaceable((key) =>
    t("editor-island-lull-hint", { key }),
  );
  return (
    <>
      {before}
      <Keyboard className="align-middle">{hotkeyKeys("AltLeft")[0]}</Keyboard>
      {after}
    </>
  );
}
