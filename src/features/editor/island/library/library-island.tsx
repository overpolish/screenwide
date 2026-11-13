// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { LibraryBig } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import { LibraryAddButton } from "./library-add-button";
import { LibraryPanel, type LibraryPanelProps } from "./library-panel";

import type { EditorIslandContent } from "../editor-island";

// The Library, brought out by the title bar's button beside the modes: the
// songs, sound effects and pictures to lay on the timeline or the picture
// (`library-panel.tsx`). Its plus adds files of the kind shown to the
// library only, wherever there is room for them; laying one down is done
// from the list.

export type LibraryIslandProps = LibraryPanelProps;

export function libraryIsland(props: LibraryIslandProps): EditorIslandContent {
  return {
    actions: <LibraryAddButton kinds={props.kinds} />,
    key: "library",
    section: <LibraryPanel {...props} />,
    title: (
      <IslandRowTitle
        glyph={<LibraryBig aria-hidden className="size-icon" />}
        title={t("editor-timeline-library")}
      />
    ),
  };
}
