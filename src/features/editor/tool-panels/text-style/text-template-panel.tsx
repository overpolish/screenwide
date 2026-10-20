// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { Text } from "../../../../primitives/text/text";
import { useFittedPanel } from "../../../popup-panel/use-fitted-panel";
import { reportPopupPanelPick } from "../../../popup-panel/use-popup-panel-picker";

import { TemplatePreviewClipView } from "./template-preview-clip";
import { TEXT_TEMPLATE_PANEL_SIZE } from "./text-template-panel-size";
import { textTemplateLabel } from "./text-templates";
import { useTemplatePreviews } from "./use-template-previews";

import type { PopupPanelTextTemplatesContent } from "../../../popup-panel/picker-content";

/**
 * The caption templates in the panel window, each a small moving picture
 * the compositor drew of a few words, in rows of three as the scene layouts
 * are. Each shows the look choosing it would set, and the chosen one the
 * captions as they are set now. The rows scroll past a few, as your own
 * templates add to them without end, starting with the chosen one in view.
 * A press reports the template to the window that opened the panel, which
 * takes it up and puts the panel away.
 */
export function TextTemplatePanel({
  content,
  focusContents,
  id,
}: {
  content: PopupPanelTextTemplatesContent;
  focusContents: boolean;
  id: string;
}) {
  const contentRef = useRef<HTMLDivElement>(null);
  useFittedPanel(contentRef, id, TEXT_TEMPLATE_PANEL_SIZE.height);
  const clips = useTemplatePreviews(content.styles);

  // Opened from the keyboard, the keys go to the chosen template.
  useEffect(() => {
    if (focusContents)
      contentRef.current
        ?.querySelector<HTMLElement>("[data-selected]")
        ?.focus();
  }, [focusContents]);

  return (
    <ScrollArea
      onInitialized={(viewport) => {
        viewport
          .querySelector("[data-selected]")
          ?.scrollIntoView({ block: "center" });
      }}
      // The scrollbar keeps out of the rounded corners.
      rootClassName="rounded-window [--scrollbar-inset:var(--radius-window)]"
    >
      <div className="w-full p-control-inset" ref={contentRef}>
        <ToggleButtonGroup
          aria-label={t("editor-panels-text-template")}
          className="grid justify-between gap-y-control"
          disallowEmptySelection
          onSelectionChange={(keys) => {
            const [chosen] = [...keys];
            if (typeof chosen === "string") reportPopupPanelPick(id, chosen);
          }}
          selectedKeys={new Set([content.selected])}
          selectionMode="single"
          // Each column a whole number of pixels, so every tile starts on one.
          style={{
            gridTemplateColumns:
              "repeat(3, round(down, calc((100% - 2 * var(--spacing-control)) / 3), 1px))",
          }}
        >
          {content.styles.map((style, index) => (
            <ToggleButton
              aria-label={textTemplateLabel(
                style.template,
                content.names[style.template],
              )}
              className={cn(
                "flex cursor-default flex-col items-stretch gap-control-inset rounded-control p-control-inset transition-colors",
                focusStyles,
                elementFocusVisible,
                content.selected === style.template &&
                  "bg-primary-surface text-primary-fg",
              )}
              id={style.template}
              key={style.template}
            >
              <TemplatePreviewClipView
                clip={clips[index]}
                still={content.still === true}
              />
              <Text className="text-center text-inherit" variant="footnote">
                {textTemplateLabel(
                  style.template,
                  content.names[style.template],
                )}
              </Text>
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </div>
    </ScrollArea>
  );
}
