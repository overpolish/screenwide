// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, useId, useState } from "react";

import { IslandFlow } from "../../../../../primitives/island/island-flow";
import { titleTemplates } from "../../../tool-panels/annotation/text/text-look";
import { useFontChoices } from "../../../tool-panels/text-style/text-fonts";
import { useTemplatePreviews } from "../../../tool-panels/text-style/use-template-previews";

import { TextFontLine, TextFontPanel } from "./text-font-picker";
import { TextPickerColumn } from "./text-picker-line";
import { TextTemplateLine, TextTemplateTiles } from "./text-template-picker";

import type { FontFamily } from "../../../../../bindings/FontFamily";
import type { TextFont } from "../../../../../bindings/TextFont";
import type { TextStyle } from "../../../../../bindings/TextStyle";
import type { TextTemplates } from "../../../recording/captions/use-text-templates";

type Picker = "accent" | "font" | "template";

/** A text's template and faces, each folded to a line, of which one at a
 * time opens its choices in a column beside the section, at its end: the
 * island grows sideways and keeps its height. Opening another line swaps
 * the column's choices. Choosing leaves them open, so one choice after
 * another can be tried; the line folds them away. The template comes
 * first, since it decides which of the lines under it there are; then how
 * the text is placed (`placement`); then its faces, the accent's offered
 * only where the template sets words apart in an accent; then the rest of
 * the section (`children`). The templates are those a text placed by hand
 * is offered, each as it would take it up. */
export function TextLookPickers({
  children,
  families,
  look,
  onLookChange,
  placement,
  still,
  templates,
}: {
  /** The section's lines under the faces. */
  children: ReactNode;
  look: TextStyle;
  onLookChange: (patch: Partial<TextStyle>) => void;
  /** The line that places the text, between its template and its faces. */
  placement: ReactNode;
  /** Whether the text never moves, as in a screenshot, so the templates'
   * previews hold on their stills. */
  still: boolean;
  templates: TextTemplates;
  /** Families beyond the system's, for a story. */
  families?: FontFamily[];
}) {
  const columnId = useId();
  const [open, setOpen] = useState<Picker | null>(null);
  const choices = useFontChoices(families);
  const { styles } = titleTemplates(templates);
  const clips = useTemplatePreviews(styles);
  const hasAccent = look.accent !== "none";
  // An accent's face can't stay open once the template has no accent.
  const shown = open === "accent" && !hasAccent ? null : open;
  const pressed = (picker: Picker) => () => {
    setOpen(shown === picker ? null : picker);
  };
  const fontPanel = (font: TextFont, key: "accentFont" | "font") => (
    <TextFontPanel
      choices={choices}
      font={font}
      onChange={(next) => {
        onLookChange({ [key]: next });
      }}
    />
  );
  const content =
    shown === "template" ? (
      <TextTemplateTiles
        clips={clips}
        names={templates.names}
        onChoose={onLookChange}
        selected={look.template}
        still={still}
        styles={styles}
      />
    ) : shown === "font" ? (
      fontPanel(look.font, "font")
    ) : shown === "accent" ? (
      fontPanel(look.accentFont, "accentFont")
    ) : null;
  return (
    // The controls keep to the start, under the row, so whatever the
    // section has beyond the island's box spills at the end, where the
    // black reveals it as it grows.
    <div className="flex">
      {/* Neither squeezed nor stretched while the island's box catches up
       * with the column, wider or narrower than what it holds: the controls
       * keep their layout and place, and the black opens or closes beside
       * them. Not stretched down either (`self-start`): a stretched item's
       * height counts as fixed, and a scroll area inside sizing itself to
       * its parent, as the colour grid's does, would be cut short. The
       * column is what stretches, to the controls' height. Lines that come
       * and go with the template do so in place, and the lines under them
       * glide to their places (`IslandFlow`). */}
      <IslandFlow className="flex shrink-0 flex-col gap-separate self-start">
        <TextTemplateLine
          clip={clips[styles.findIndex((s) => s.template === look.template)]}
          controls={columnId}
          isOpen={shown === "template"}
          names={templates.names}
          onPress={pressed("template")}
          selected={look.template}
        />
        {placement}
        <IslandFlow className="flex flex-col gap-inline">
          <TextFontLine
            choices={choices}
            controls={columnId}
            font={look.font}
            isOpen={shown === "font"}
            onPress={pressed("font")}
          />
          {hasAccent ? (
            <TextFontLine
              choices={choices}
              controls={columnId}
              font={look.accentFont}
              isAccent
              isOpen={shown === "accent"}
              onPress={pressed("accent")}
            />
          ) : null}
        </IslandFlow>
        {children}
      </IslandFlow>
      <TextPickerColumn
        content={content}
        contentKey={shown ?? ""}
        id={columnId}
      />
    </div>
  );
}
