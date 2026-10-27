// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { STORY_TEXT_TEMPLATES } from "../../../../features/editor/tool-panels/text-style/text-template-fixtures";
import { textTemplateLabel } from "../../../../features/editor/tool-panels/text-style/text-templates";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";

// The caption templates, three to a row as the app's template panel has
// them, scrolling past three rows. The template decides how a caption shows
// its words: a line in a box, the spoken word lit, words rising as they are
// said. In the app each tile plays the compositor's preview of its look; a
// story has no compositor, so here the tile is blank above its name.

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
      // Only the grid scrolls to the chosen tile: `scrollIntoView` would
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
      <ToggleButtonGroup
        aria-label="Caption template"
        className="grid grid-cols-3 gap-1"
        disallowEmptySelection
        onSelectionChange={(keys) => {
          for (const key of keys) onChange(String(key));
        }}
        selectedKeys={[value]}
        selectionMode="single"
      >
        {styles.map(({ template }) => {
          const label = textTemplateLabel(template, names[template]);
          return (
            <ToggleButton
              aria-label={label}
              className={cn(
                "flex cursor-default flex-col gap-1 rounded-[10px] p-1 text-caption text-fg-secondary transition-colors hover:bg-fill data-[selected]:bg-fill-strong data-[selected]:text-fg",
                focusStyles,
                elementFocusVisible,
              )}
              id={template}
              key={template}
            >
              <span
                aria-hidden
                className="block aspect-video w-full rounded-[6px] bg-fill"
              />
              <span className="truncate text-center">{label}</span>
            </ToggleButton>
          );
        })}
      </ToggleButtonGroup>
    </ScrollArea>
  );
}
