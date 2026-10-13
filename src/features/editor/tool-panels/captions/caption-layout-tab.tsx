// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { NumberField } from "../../../../components/base/input-fields/number-field";
import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { PositionGrid } from "../../../../components/base/position-grid/position-grid";
import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";
import {
  isEmptyLayout,
  isSameLayout,
  layoutOf,
} from "../../recording/captions/caption-layout";
import {
  CAPTION_SIZE_PERCENT,
  CAPTION_WORDS,
  CaptionsPatch,
} from "../../recording/captions/recording-captions";
import { CaptionTemplates } from "../../recording/captions/use-caption-templates";

import type { CaptionLayout } from "../../../../bindings/CaptionLayout";
import type { CaptionLines } from "../../../../bindings/CaptionLines";
import type { CaptionPosition } from "../../../../bindings/CaptionPosition";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";

const lineOptions = (): { id: CaptionLines; label: string }[] => [
  { id: "one", label: t("editor-panels-captions-one-line") },
  { id: "two", label: t("editor-panels-captions-two-lines") },
];

/** The nine places, in reading order, as the grid lays them out. */
const positionOptions = (): { id: CaptionPosition; label: string }[] => [
  { id: "top-left", label: t("editor-panels-captions-position-top-left") },
  { id: "top", label: t("editor-panels-captions-position-top") },
  { id: "top-right", label: t("editor-panels-captions-position-top-right") },
  { id: "left", label: t("editor-panels-captions-position-left") },
  { id: "center", label: t("editor-panels-captions-position-center") },
  { id: "right", label: t("editor-panels-captions-position-right") },
  {
    id: "bottom-left",
    label: t("editor-panels-captions-position-bottom-left"),
  },
  { id: "bottom", label: t("editor-panels-captions-position-bottom") },
  {
    id: "bottom-right",
    label: t("editor-panels-captions-position-bottom-right"),
  },
];

/**
 * The Layout tab: how large the type is, where on the picture it sits, its
 * case, and how many words and lines a caption holds. A template brings all
 * of them; one changed here is kept as it is when another template is
 * chosen, and the rest follow the template. Reset lets every one follow the
 * template again; Set as default keeps the ones changed for new captions.
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
  layout: CaptionLayout;
  onChange: (patch: CaptionsPatch) => void;
  onDefaultsChange: (layout: CaptionLayout) => void;
  style: CaptionStyle;
  templates: CaptionTemplates;
}) {
  const set = (change: CaptionLayout) => {
    onChange({ layout: { ...layout, ...change }, style: change });
  };
  const template = templates.styles.find(
    (each) => each.template === style.template,
  );

  return (
    <>
      <ControlRow title={t("editor-panels-size")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label={t("editor-panels-size")}
              className="w-48"
              isDisabled={isDisabled}
              maxValue={CAPTION_SIZE_PERCENT.maximum}
              minValue={CAPTION_SIZE_PERCENT.minimum}
              onChange={(sizePercent) => {
                set({ sizePercent });
              }}
              rightSection="%"
              step={5}
              value={style.sizePercent}
            />
          </div>
        )}
      </ControlRow>

      <ControlRow title={t("editor-panels-position")}>
        {(controlProps) => (
          <PositionGrid
            aria-labelledby={controlProps["aria-labelledby"]}
            isDisabled={isDisabled}
            items={positionOptions()}
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

      <ControlRow title={t("editor-panels-captions-lines")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <PillGroup
              aria-label={t("editor-panels-captions-lines")}
              display="label"
              isDisabled={isDisabled}
              items={lineOptions()}
              onSelectionChange={(lines) => {
                set({ lines: lines as CaptionLines });
              }}
              selected={style.lines}
            />
          </div>
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
