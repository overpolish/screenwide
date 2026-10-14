// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useRef } from "react";

import { ListBoxItem } from "../../../../components/base/listbox-item/listbox-item";
import { Select } from "../../../../components/base/select/select";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import {
  boundsAnchor,
  pointerAnchor,
} from "../../../popup-panel/use-popup-menu";
import { usePopupPanelPicker } from "../../../popup-panel/use-popup-panel-picker";
import { withLayout } from "../../recording/captions/caption-layout";

import { CAPTION_FIELD_WIDTH } from "./caption-field-widths";
import { captionTemplateLabel } from "./caption-templates";
import { useCaptionTemplateMenu } from "./use-caption-template-menu";

import type { CaptionLayout } from "../../../../bindings/CaptionLayout";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplate } from "../../../../bindings/CaptionTemplate";
import type { CaptionTemplates } from "../../recording/captions/use-caption-templates";

const PICKER_ID = "caption-templates";
/** Three tiles across; the panel fits its height to the rows. */
const PICKER_SIZE = { height: 400, width: 380 };

/**
 * The template the captions take their look from, named on a button that
 * opens every template's moving preview to choose from. Each shows its saved
 * look with the layout options set by hand, as choosing it would set the
 * captions, and the chosen one the captions as they are set now. A choice
 * takes the template's style up whole, but for the layout set by hand; a
 * right click on the button, or the menu key, offers to put the chosen
 * template back to how it ships.
 */
export function CaptionTemplateRow({
  isDisabled,
  layout,
  onChoose,
  onResetToOriginal,
  style,
  templates,
}: {
  isDisabled: boolean;
  layout: CaptionLayout;
  onChoose: (style: CaptionStyle) => void;
  onResetToOriginal: (template: CaptionTemplate) => void;
  style: CaptionStyle;
  templates: CaptionTemplates;
}) {
  const openMenu = useCaptionTemplateMenu(onResetToOriginal);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const shown = useMemo(
    () =>
      templates.styles.map((template) =>
        template.template === style.template
          ? style
          : withLayout(layout, template),
      ),
    [layout, style, templates.styles],
  );
  const picker = usePopupPanelPicker({
    closesOnPick: true,
    content: {
      kind: "caption-templates",
      selected: style.template,
      styles: shown,
    },
    id: PICKER_ID,
    label: t("editor-panels-captions-template"),
    onPick: (template) => {
      const chosen = templates.styles.find(
        (each) => each.template === template,
      );
      if (chosen && chosen.template !== style.template) onChoose(chosen);
    },
  });
  const chosen = {
    id: style.template,
    label: captionTemplateLabel(style.template),
  };

  return (
    <ControlRow title={t("editor-panels-captions-template-picker")}>
      {(controlProps) => (
        <div
          className={CAPTION_FIELD_WIDTH}
          data-popup-panel-trigger={PICKER_ID}
          onContextMenu={(event) => {
            event.preventDefault();
            if (!isDisabled)
              void openMenu(
                style.template,
                pointerAnchor(event.clientX, event.clientY),
              );
          }}
          onKeyDown={(event) => {
            const trigger = triggerRef.current;
            const isMenuKey =
              event.key === "ContextMenu" ||
              (event.key === "F10" && event.shiftKey);
            if (!isMenuKey || isDisabled || !trigger) return;
            event.preventDefault();
            void openMenu(
              style.template,
              boundsAnchor(trigger.getBoundingClientRect()),
            );
          }}
          onPointerDown={(event) => {
            event.stopPropagation();
          }}
        >
          <Select
            aria-labelledby={controlProps["aria-labelledby"]}
            className="w-full"
            clearable={false}
            isDisabled={isDisabled}
            isOpen={picker.isOpen}
            items={[chosen]}
            onPress={(event) => {
              const trigger = triggerRef.current;
              if (trigger)
                void picker.toggle(
                  trigger,
                  PICKER_SIZE,
                  ["keyboard", "virtual"].includes(event.pointerType),
                );
            }}
            renderValue={(item) =>
              item ? (
                <span className="flex w-full items-center">
                  <span className="truncate">{item.label}</span>
                </span>
              ) : null
            }
            standalone
            triggerRef={triggerRef}
            value={chosen.id}
          >
            {(item: typeof chosen) => (
              <ListBoxItem id={item.id} textValue={item.label}>
                {item.label}
              </ListBoxItem>
            )}
          </Select>
        </div>
      )}
    </ControlRow>
  );
}
