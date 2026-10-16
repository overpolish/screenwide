// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ellipsis } from "lucide-react";
import { useMemo, useRef } from "react";
import { useCollator } from "react-aria";

import { IconButton } from "../../../../components/base/button/icon-button";
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

import { TEXT_FIELD_WIDTH } from "./text-field-widths";
import { TEXT_TEMPLATE_PANEL_SIZE } from "./text-template-panel-size";
import { textTemplateLabel } from "./text-templates";
import { useTextTemplateActions } from "./use-text-template-actions";

import type { TextLayout } from "../../../../bindings/TextLayout";
import type { TextStyle } from "../../../../bindings/TextStyle";
import type { TextTemplateId } from "../../../../bindings/TextTemplateId";
import type { TextTemplates } from "../../recording/captions/use-text-templates";

const PICKER_ID = "text-templates";

/**
 * The template the captions take their look from, named on a button that
 * opens every template's moving preview to choose from. Each shows its saved
 * look with the layout options set by hand, as choosing it would set the
 * captions, and the chosen one the captions as they are set now. A choice
 * takes the template's style up whole, but for the layout set by hand. The
 * button beside it, a right click, or the menu key opens the template's
 * menu: import one, export this one, remove one of yours, or put it back to
 * how it ships. One imported is taken up at once.
 */
export function TextTemplateRow({
  isDisabled,
  layout,
  onChoose,
  onResetToOriginal,
  still = false,
  style,
  templates,
}: {
  isDisabled: boolean;
  layout: TextLayout;
  onChoose: (style: TextStyle) => void;
  onResetToOriginal: (template: TextTemplateId) => void;
  style: TextStyle;
  templates: TextTemplates;
  /** Whether the text never moves, so the tiles hold on their stills. */
  still?: boolean;
}) {
  const openMenu = useTextTemplateActions({
    onImported: (template) => {
      onChoose(template.style);
    },
    onResetToOriginal,
    sources: templates.sources,
  });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLButtonElement>(null);
  // Every template by its name as shown, the app's own and yours together,
  // in the order the app's language sorts them.
  const collator = useCollator({ sensitivity: "base" });
  const shown = useMemo(() => {
    const label = (template: TextStyle) =>
      textTemplateLabel(template.template, templates.names[template.template]);
    return templates.styles
      .map((template) =>
        template.template === style.template
          ? style
          : withLayout(layout, template),
      )
      .sort((a, b) => collator.compare(label(a), label(b)));
  }, [collator, layout, style, templates.names, templates.styles]);
  const picker = usePopupPanelPicker({
    closesOnPick: true,
    content: {
      kind: "text-templates",
      names: templates.names,
      selected: style.template,
      still,
      styles: shown,
    },
    id: PICKER_ID,
    label: t("editor-panels-text-template"),
    onPick: (template) => {
      const chosen = templates.styles.find(
        (each) => each.template === template,
      );
      if (chosen && chosen.template !== style.template) onChoose(chosen);
    },
  });
  const chosen = {
    id: style.template,
    label: textTemplateLabel(style.template, templates.names[style.template]),
  };

  return (
    <ControlRow title={t("editor-panels-text-template-picker")}>
      {(controlProps) => (
        <div
          className={`flex items-center gap-control ${TEXT_FIELD_WIDTH}`}
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
            className="min-w-0 flex-1"
            clearable={false}
            isDisabled={isDisabled}
            isOpen={picker.isOpen}
            items={[chosen]}
            onPress={(event) => {
              const trigger = triggerRef.current;
              if (trigger)
                void picker.toggle(
                  trigger,
                  TEXT_TEMPLATE_PANEL_SIZE,
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
          <IconButton
            aria-label={t("editor-panels-template-actions")}
            isDisabled={isDisabled}
            onPress={() => {
              const menu = menuRef.current;
              if (menu)
                void openMenu(
                  style.template,
                  boundsAnchor(menu.getBoundingClientRect()),
                );
            }}
            ref={menuRef}
          >
            <Ellipsis />
          </IconButton>
        </div>
      )}
    </ControlRow>
  );
}
