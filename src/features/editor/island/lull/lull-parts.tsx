// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";

import type { LullIslandProps } from "./lull-island";

// A lull's choices, on the line under its row: how it plays, and its trim.
// How it plays is a choice, so it is a list with a tick; a trim is done at
// once, so it is a one-shot action at the end of its own row. At the
// island's foot, the key that turns dragging its end from a retime into a
// trim.

/** The lull island's width, set by its line: a black 260 px wide, which
 * holds the longest way it plays, so a retime coming or going leaves the
 * island as it is. */
const LULL_LINE_WIDTH = "w-63";

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
    // Inset as the row's content is, with room beneath before the hint, at
    // the island's width. A retime coming or going moves the trim with the
    // black (`IslandFlow`).
    <IslandFlow
      className={cn(
        "flex flex-col gap-separate px-1 pb-inset-sm",
        LULL_LINE_WIDTH,
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
    </IslandFlow>
  );
}
