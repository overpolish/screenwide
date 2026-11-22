// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";

import type { BeatIslandProps } from "./beat-island";

// How a beat plays, on the line under its row, as a lull's is: a list with
// a tick, as recorded or at the speed its end was dragged to with ⌥. The
// retime is made on the strip, so it is a choice here only once made, and
// choosing to play as recorded takes it back.

/** The beat island's line, as wide as the lull's (`lull-parts.tsx`), which
 * holds the longest way it plays, so a retime coming or going leaves the
 * island as it is. */
const BEAT_LINE_WIDTH = "w-63";

export function BeatPlays({
  onPlayAsRecorded,
  speed,
}: Pick<BeatIslandProps, "onPlayAsRecorded" | "speed">) {
  const format = useNumberFormatter({
    maximumFractionDigits: 1,
    minimumFractionDigits: 1,
  });
  const isRetimed = speed !== 1;
  const retimed =
    speed > 1
      ? t("editor-island-beat-retimed-faster", { speed: format.format(speed) })
      : t("editor-island-beat-retimed-slower", { speed: format.format(speed) });
  return (
    <IslandParts
      className={cn("flex flex-col px-1 pb-inset-sm", BEAT_LINE_WIDTH)}
    >
      <ListBox
        aria-label={t("editor-island-beat-plays")}
        disallowEmptySelection
        onSelectionChange={(keys) => {
          if (keys === "all") return;
          const [choice] = keys;
          if (choice === "recorded" && isRetimed) onPlayAsRecorded();
        }}
        selectedKeys={[isRetimed ? "retimed" : "recorded"]}
        selectionMode="single"
      >
        <ListBoxItem id="recorded">
          {t("editor-island-beat-as-recorded")}
        </ListBoxItem>
        {isRetimed ? <ListBoxItem id="retimed">{retimed}</ListBoxItem> : null}
      </ListBox>
    </IslandParts>
  );
}
