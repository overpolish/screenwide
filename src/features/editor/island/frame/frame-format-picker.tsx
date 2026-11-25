// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Header,
  ListBoxSection as AriaListBoxSection,
} from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";
import { revealChosen } from "../../../../primitives/scroll-area/reveal-chosen";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { ShapeGlyph } from "../../glyphs/shape-glyph";
import {
  FRAME_SIZE_PRESETS,
  frameSizePreset,
  frameSizePresetName,
  frameSizePresetPlatform,
} from "../../tools/frame/frame-sizes";
import { TextPickerLine } from "../annotation/text/text-picker-line";

// The template a recording's main canvas is made for, folded to a line at
// the foot of the Frame island, opposite Original size, that opens the
// templates in the island's column (`IslandPickerColumn`), where the Library
// opens too. Choosing one sizes the main canvas and keeps it clear of that
// app's interface (`main-made-for.ts`), and folds the column away; a size
// typed in makes it Custom again.

const CUSTOM = "custom";

type Size = { height: number; width: number };

/** The line: the shape and the name of what the main canvas is made for. */
export function FrameFormatLine({
  controls,
  isOpen,
  madeFor,
  onPress,
  size,
}: {
  controls: string;
  isOpen: boolean;
  madeFor: string | null;
  onPress: () => void;
  /** The main canvas's, which the glyph draws the shape of. */
  size: Size;
}) {
  const preset = frameSizePreset(madeFor);
  const name = preset
    ? frameSizePresetName(preset.id)
    : t("editor-frame-made-for-custom");
  return (
    <TextPickerLine
      controls={controls}
      isCompact
      isOpen={isOpen}
      label={t("editor-frame-made-for-current", { format: name })}
      onPress={onPress}
    >
      <ShapeGlyph className="size-icon shrink-0" size={size} />
      <span className="truncate">{name}</span>
    </TextPickerLine>
  );
}

/** The templates, platform by platform under Custom, each with its shape
 * and size; the one the main canvas is made for chosen. */
export function FrameFormatChoices({
  madeFor,
  onChoose,
}: {
  madeFor: string | null;
  onChoose: (preset: string | null) => void;
}) {
  const platforms = [
    ...new Set(
      FRAME_SIZE_PRESETS.map((preset) => frameSizePresetPlatform(preset.id)),
    ),
  ];
  return (
    <ScrollArea onInitialized={revealChosen} rootClassName="h-full min-h-0">
      <ListBox
        aria-label={t("editor-frame-made-for")}
        disallowEmptySelection
        onSelectionChange={(keys) => {
          if (keys === "all") return;
          const [key] = [...keys];
          onChoose(key === CUSTOM ? null : String(key));
        }}
        selectedKeys={[madeFor ?? CUSTOM]}
        selectionMode="single"
      >
        <ListBoxItem id={CUSTOM} textValue={t("editor-frame-made-for-custom")}>
          {t("editor-frame-made-for-custom")}
        </ListBoxItem>
        {platforms.map((platform) => (
          <AriaListBoxSection
            className="flex flex-col"
            id={platform}
            key={platform}
          >
            <Header>
              <span className="block px-inset-sm pt-separate pb-inline text-caption text-fg-secondary">
                {platform}
              </span>
            </Header>
            {FRAME_SIZE_PRESETS.filter(
              (preset) => frameSizePresetPlatform(preset.id) === platform,
            ).map((preset) => {
              const name = frameSizePresetName(preset.id);
              return (
                <ListBoxItem id={preset.id} key={preset.id} textValue={name}>
                  <span className="flex items-center gap-inline">
                    <ShapeGlyph className="size-icon shrink-0" size={preset} />
                    <span className="min-w-0 flex-1 truncate">{name}</span>
                    <span className="shrink-0 text-fg-secondary">
                      {t("editor-frame-format-size", preset)}
                    </span>
                  </span>
                </ListBoxItem>
              );
            })}
          </AriaListBoxSection>
        ))}
      </ListBox>
    </ScrollArea>
  );
}
