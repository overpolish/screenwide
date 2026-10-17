// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ListBox,
  ListBoxItem,
  ListBoxSection,
  Selection,
} from "react-aria-components";

import { ControlRow } from "../../../../../components/shared/control-row/control-row";
import { t } from "../../../../../i18n/i18n";
import {
  cn,
  elementFocusVisible,
  focusStyles,
} from "../../../../../lib/styling";

import { isChosenWord, textWordLines } from "./text-words";

import type { TextSpan } from "../../../../../bindings/TextSpan";
import type { TextStyle } from "../../../../../bindings/TextStyle";
import type { ToolPanelPatch } from "../../tool-panel-patch";
// A word reads as text until it is chosen, when it takes the accent's fill;
// the pointer over it gives it a quiet one, as a caption's words take.
const wordClass = cn(
  "cursor-default rounded-swatch px-tight outline-none",
  "data-[hovered]:bg-fill-secondary",
  "data-[selected]:bg-primary-surface data-[selected]:text-primary-fg",
  "data-[disabled]:text-content-fg-tertiary",
  focusStyles,
  elementFocusVisible,
);

/**
 * The words of a text, laid out in its lines, any of which a press sets
 * apart in the template's second appearance or takes back: its accent for a
 * template that sets a key word apart, its highlight for one that
 * highlights words. Nothing is set apart until a word is chosen.
 */
export function AnnotationTextWordsRow({
  change,
  emphasis,
  isLocked,
  look,
  text,
}: {
  change: (patch: ToolPanelPatch) => void;
  emphasis: TextSpan[];
  isLocked: boolean;
  look: TextStyle;
  text: string;
}) {
  const lines = textWordLines(text);
  const words = lines.flat();
  const key = (span: TextSpan) => String(span.start);
  const selected = words.filter((word) => isChosenWord(emphasis, word));
  const choose = (keys: Selection) => {
    const chosen =
      keys === "all" ? words : words.filter((word) => keys.has(key(word.span)));
    change({ annotationEmphasis: chosen.map((word) => word.span) });
  };
  return (
    <ControlRow
      className="flex-col items-stretch gap-control"
      controlClassName="w-full"
      description={t("editor-panels-text-chosen-words-description")}
      title={
        look.accent === "word"
          ? t("editor-panels-text-key-words")
          : t("editor-panels-text-highlighted-words")
      }
    >
      {(controlProps) => (
        <ListBox
          {...controlProps}
          className="flex w-full flex-col gap-tight rounded-control bg-fill-tertiary p-control-inset text-body text-content-fg outline-none"
          dir="auto"
          disabledKeys={isLocked ? words.map((word) => key(word.span)) : []}
          layout="grid"
          onSelectionChange={choose}
          selectedKeys={new Set(selected.map((word) => key(word.span)))}
          selectionBehavior="toggle"
          selectionMode="multiple"
        >
          {lines.map((line) => (
            <ListBoxSection
              className="flex flex-wrap gap-tight"
              id={`line-${key(line[0].span)}`}
              key={key(line[0].span)}
            >
              {line.map((word) => (
                <ListBoxItem
                  className={wordClass}
                  id={key(word.span)}
                  key={key(word.span)}
                  textValue={word.text}
                >
                  {word.text}
                </ListBoxItem>
              ))}
            </ListBoxSection>
          ))}
        </ListBox>
      )}
    </ControlRow>
  );
}
