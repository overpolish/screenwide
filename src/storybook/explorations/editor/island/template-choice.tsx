// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { STORY_TEXT_TEMPLATES } from "../../../../features/editor/tool-panels/text-style/text-template-fixtures";
import { textTemplateLabel } from "../../../../features/editor/tool-panels/text-style/text-templates";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { TileChoice } from "../../../../primitives/tile-choice/tile-choice";

// The caption templates, scrolling past three rows, opening on the chosen
// one. The template decides how a caption shows its words: a line in a box,
// the spoken word lit, words rising as they are said. In the app each tile
// plays the compositor's preview of its look; a story has no compositor, so
// here the tile is blank above its name.

const { names, styles } = STORY_TEXT_TEMPLATES;

export function TemplateChoice({
  onChange,
  value,
}: {
  onChange: (template: string) => void;
  value: string;
}) {
  return (
    <ScrollArea
      className="p-inset-sm"
      constrainHeight
      // Only the tiles scroll to the chosen one: `scrollIntoView` would
      // scroll the workspace under the island too.
      onInitialized={(viewport) => {
        const chosen = viewport.querySelector<HTMLElement>("[data-selected]");
        if (chosen)
          viewport.scrollTop =
            chosen.offsetTop -
            (viewport.clientHeight - chosen.offsetHeight) / 2;
      }}
      rootClassName="-my-inset-sm max-h-60 w-72"
    >
      <TileChoice
        aria-label="Caption template"
        items={styles.map(({ template }) => ({
          id: template,
          label: textTemplateLabel(template, names[template]),
        }))}
        onChange={onChange}
        value={value}
      />
    </ScrollArea>
  );
}
