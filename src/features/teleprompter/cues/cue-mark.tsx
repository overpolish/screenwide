// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { ImageIcon } from "lucide-react";

import { t } from "../../../i18n/i18n";
import {
  boundsAnchor,
  type PopupMenuAnchor,
} from "../../popup-panel/use-popup-menu";

// A cue's mark beside its word, drawn among the words under the script's
// field. Placed absolutely with no top of its own, it stays on its word's
// line, which only an inline box does: a block one would drop to the next.
// Its inline end puts it in the room the column keeps at its end for
// marks, and it stands over the field so it can be pressed. It opens the
// word's menu, as a right click on the word does; like the words it is
// drawn among, it is left out of the tab order and assistive technology.
export function CueMark({
  onPress,
  title,
}: {
  onPress: (anchor: PopupMenuAnchor) => void;
  title: string;
}) {
  return (
    <button
      aria-label={t("teleprompter-cue-mark", { title })}
      className="absolute end-1 z-10 mt-[calc((1lh-1.5rem)/2)] inline-grid size-6 place-items-center rounded-full bg-fill-strong text-fg-secondary transition-colors hover:text-fg"
      onClick={(event) => {
        onPress(boundsAnchor(event.currentTarget.getBoundingClientRect()));
      }}
      tabIndex={-1}
      type="button"
    >
      <ImageIcon aria-hidden className="size-icon-sm" />
    </button>
  );
}
