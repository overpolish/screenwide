// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { Text } from "../../../../components/base/text/text";
import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { useFittedPanel } from "../../../popup-panel/use-fitted-panel";
import { reportPopupPanelPick } from "../../../popup-panel/use-popup-panel-picker";

import { CaptionPreviewStripView } from "./caption-preview-strip";
import { captionTemplateLabel } from "./caption-templates";
import { useCaptionPreviews } from "./use-caption-previews";

import type { PopupPanelCaptionTemplatesContent } from "../../../popup-panel/picker-content";

/**
 * The caption templates in the panel window, each a small moving picture
 * the compositor drew of a few words, in rows of three as the scene layouts
 * are. Each shows the look choosing it would set, and the chosen one the
 * captions as they are set now. A press reports the template to the window
 * that opened the panel, which takes it up and puts the panel away.
 */
export function CaptionTemplatePanel({
  content,
  focusContents,
  id,
}: {
  content: PopupPanelCaptionTemplatesContent;
  focusContents: boolean;
  id: string;
}) {
  const contentRef = useRef<HTMLDivElement>(null);
  useFittedPanel(contentRef, id);
  const strips = useCaptionPreviews(content.styles);

  // Opened from the keyboard, the keys go to the chosen template.
  useEffect(() => {
    if (focusContents)
      contentRef.current
        ?.querySelector<HTMLElement>("[data-selected]")
        ?.focus();
  }, [focusContents]);

  return (
    <div className="w-full p-control-inset" ref={contentRef}>
      <ToggleButtonGroup
        aria-label={t("editor-panels-captions-template")}
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
            aria-label={captionTemplateLabel(
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
            <CaptionPreviewStripView strip={strips?.[index]} />
            <Text className="text-center text-inherit" variant="footnote">
              {captionTemplateLabel(
                style.template,
                content.names[style.template],
              )}
            </Text>
          </ToggleButton>
        ))}
      </ToggleButtonGroup>
    </div>
  );
}
