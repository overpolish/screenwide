// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { revealChosen } from "../../../../../primitives/scroll-area/reveal-chosen";
import { ScrollArea } from "../../../../../primitives/scroll-area/scroll-area";
import { TileChoice } from "../../../../../primitives/tile-choice/tile-choice";
import { TemplatePreviewClipView } from "../../../tool-panels/text-style/template-preview-clip";
import { textTemplateLabel } from "../../../tool-panels/text-style/text-templates";

import { TextPickerLine } from "./text-picker-line";

import type { TemplatePreviewClip } from "../../../../../bindings/TemplatePreviewClip";
import type { TextStyle } from "../../../../../bindings/TextStyle";
import type { TextTemplateId } from "../../../../../bindings/TextTemplateId";

/** Each template's name, as its manifest gives it. */
type TemplateNames = Partial<Record<TextTemplateId, string>>;

/** The template a text takes its look from, folded to a line with its
 * preview, held still, and its name. */
export function TextTemplateLine({
  clip,
  controls,
  isOpen,
  names,
  onPress,
  selected,
}: {
  /** The chosen template's preview, once the compositor has drawn it. */
  clip: TemplatePreviewClip | undefined;
  controls: string;
  isOpen: boolean;
  names: TemplateNames;
  onPress: () => void;
  selected: TextTemplateId;
}) {
  const name = textTemplateLabel(selected, names[selected]);
  return (
    <TextPickerLine
      controls={controls}
      isOpen={isOpen}
      label={t("editor-island-text-template-current", { name })}
      onPress={onPress}
    >
      <span className="w-10 shrink-0">
        <TemplatePreviewClipView clip={clip} still />
      </span>
      <span className="truncate">{name}</span>
    </TextPickerLine>
  );
}

/** The templates as tiles, two to a row so each preview is large enough to
 * read, each its moving preview over its name, scrolling within the column
 * they open in. Each preview is a clip the compositor drew, so a tile
 * without one keeps its room and shows its name. A still text's tiles hold
 * on their stills. */
export function TextTemplateTiles({
  clips,
  names,
  onChoose,
  selected,
  still,
  styles,
}: {
  /** The previews of `styles`, in order, as the compositor draws them. */
  clips: readonly (TemplatePreviewClip | undefined)[];
  names: TemplateNames;
  onChoose: (style: TextStyle) => void;
  selected: TextTemplateId;
  still: boolean;
  /** The templates on offer, each as choosing it would set the text. */
  styles: TextStyle[];
}) {
  return (
    <ScrollArea
      onInitialized={(viewport) => {
        revealChosen(viewport, "[data-selected]");
      }}
    >
      <TileChoice
        aria-label={t("editor-island-text-template")}
        columns={2}
        items={styles.map((style, index) => ({
          id: style.template,
          label: textTemplateLabel(style.template, names[style.template]),
          picture: (
            <TemplatePreviewClipView clip={clips[index]} still={still} />
          ),
        }))}
        onChange={(template) => {
          const style = styles.find((each) => each.template === template);
          if (style) onChoose(style);
        }}
        value={selected}
      />
    </ScrollArea>
  );
}
