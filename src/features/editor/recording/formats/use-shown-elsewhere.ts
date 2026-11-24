// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useListFormatter } from "react-aria";

import { listedFormats } from "./format-name";
import { useFrameFormats } from "./frame-formats-context";

/** For something the format in hand leaves out, by id, the formats it
 * shows in, named in a list for its accessible name; null for anything
 * shown here. */
export function useShownElsewhere() {
  const formats = useFrameFormats();
  const list = useListFormatter({ style: "long", type: "conjunction" });
  return (id: string) => {
    const snapshot = formats?.snapshot;
    const ids = snapshot?.differs?.hidden.shownIn[id];
    if (!snapshot || !ids) return null;
    return list.format(
      listedFormats(snapshot)
        .filter((format) => ids.includes(format.id))
        .map((format) => format.name),
    );
  };
}
