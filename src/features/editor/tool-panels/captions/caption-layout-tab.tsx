// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { PositionGrid } from "../../../../primitives/position-grid/position-grid";
import { Switch } from "../../../../primitives/switch/switch";
import {
  isEmptyLayout,
  isSameLayout,
  layoutOf,
} from "../../recording/captions/caption-layout";
import {
  CAPTION_WORDS,
  CaptionsPatch,
} from "../../recording/captions/recording-captions";
import { TextTemplates } from "../../recording/captions/use-text-templates";
import { placementOptions } from "../placement-options";

import { CaptionSizeRow } from "./caption-size-row";

import type { TextLayout } from "../../../../bindings/TextLayout";
import type { TextLines } from "../../../../bindings/TextLines";
import type { TextStyle } from "../../../../bindings/TextStyle";

const lineOptions = (): { id: TextLines; label: string }[] => [
  { id: "one", label: t("editor-panels-captions-one-line") },
  { id: "two", label: t("editor-panels-captions-two-lines") },
];

/**
 * The Layout tab: how large the type and its key word are, a step from the
 * template's own sizes, where on the picture the type sits, its case, and
 * how many words and lines a caption holds. A template brings all of them;
 * one changed here is kept as it is when another template is chosen, and
 * the rest follow the template. Reset lets every one follow the template
 * again; Set as default keeps the ones changed for new captions.
 */
export function CaptionLayoutTab({
  isDisabled,
  layout,
  onChange,
  onDefaultsChange,
  style,
  templates,
}: {
  isDisabled: boolean;
  layout: TextLayout;
  onChange: (patch: CaptionsPatch) => void;
  onDefaultsChange: (layout: TextLayout) => void;
  style: TextStyle;
  templates: TextTemplates;
}) {
  const set = (change: TextLayout) => {
    onChange({ layout: { ...layout, ...change }, style: change });
  };
  const template = templates.styles.find(
    (each) => each.template === style.template,
  );

  return (
    <>
      <CaptionSizeRow
        isDisabled={isDisabled}
        onChange={(size) => {
          set({ size });
        }}
        size={style.size}
        title={t("editor-panels-size")}
      />

      {/* Only a template that sets a key word apart has one to size. */}
      {style.accent === "word" ? (
        <CaptionSizeRow
          isDisabled={isDisabled}
          onChange={(accentSize) => {
            set({ accentSize });
          }}
          size={style.accentSize}
          title={t("editor-panels-captions-accent-size")}
        />
      ) : null}

      <ControlRow title={t("editor-panels-position")}>
        {(controlProps) => (
          <PositionGrid
            aria-labelledby={controlProps["aria-labelledby"]}
            isDisabled={isDisabled}
            items={placementOptions()}
            onChange={(position) => {
              set({ position });
            }}
            value={style.position}
          />
        )}
      </ControlRow>

      <ControlRow title={t("editor-panels-captions-max-words")}>
        {(controlProps) => (
          <NumberField
            {...controlProps}
            className="w-24"
            isDisabled={isDisabled}
            maxValue={CAPTION_WORDS.maximum}
            minValue={CAPTION_WORDS.minimum}
            onChange={(maxWords) => {
              set({ maxWords });
            }}
            step={1}
            value={style.maxWords}
          />
        )}
      </ControlRow>

      <ControlRow title={t("editor-panels-captions-all-caps")}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isDisabled}
            isSelected={style.allCaps}
            onChange={(allCaps) => {
              set({ allCaps });
            }}
          />
        )}
      </ControlRow>

      {/* A stack or a staircase breaks its own lines, so the count only
       * holds for captions set in lines. */}
      {style.arrangement === "lines" ? (
        <ControlRow title={t("editor-panels-captions-lines")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("editor-panels-captions-lines")}
                isDisabled={isDisabled}
                items={lineOptions()}
                onSelectionChange={(lines) => {
                  set({ lines: lines as TextLines });
                }}
                selected={style.lines}
              />
            </div>
          )}
        </ControlRow>
      ) : null}

      <div className="flex flex-wrap justify-end gap-control">
        <Button
          isDisabled={isDisabled || !template || isEmptyLayout(layout)}
          onPress={() => {
            if (template) onChange({ layout: {}, style: layoutOf(template) });
          }}
        >
          {t("editor-panels-reset")}
        </Button>
        <Button
          isDisabled={
            isDisabled || isSameLayout(layout, templates.defaultLayout)
          }
          onPress={() => {
            onDefaultsChange(layout);
          }}
        >
          {t("editor-panels-captions-set-default")}
        </Button>
      </div>
    </>
  );
}
