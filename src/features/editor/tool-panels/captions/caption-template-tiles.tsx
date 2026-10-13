// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useMemo, useRef } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { ScrollArea } from "../../../../components/base/scroll-area/scroll-area";
import { Text } from "../../../../components/base/text/text";
import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import {
  boundsAnchor,
  pointerAnchor,
} from "../../../popup-panel/use-popup-menu";
import { withLayout } from "../../recording/captions/caption-layout";

import { CaptionPreviewStripView } from "./caption-preview-strip";
import { captionTemplateLabel } from "./caption-templates";
import { useCaptionPreviews } from "./use-caption-previews";
import { useCaptionTemplateMenu } from "./use-caption-template-menu";

import type { CaptionLayout } from "../../../../bindings/CaptionLayout";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplate } from "../../../../bindings/CaptionTemplate";
import type { CaptionTemplates } from "../../recording/captions/use-caption-templates";

/** The rows of tiles in view at once; more templates scroll beneath them. */
const ROWS_SHOWN = 2;

/** The height `rows` rows of the tiles in `grid` take, its padding included,
 * read off the tiles as drawn: their width follows the panel's, and their
 * height the picture's shape under it. */
const rowsHeight = (grid: HTMLElement, rows: number) => {
  const tile = grid.firstElementChild?.getBoundingClientRect().height ?? 0;
  const style = getComputedStyle(grid);
  return (
    tile * rows +
    parseFloat(style.rowGap) * (rows - 1) +
    parseFloat(style.paddingTop) +
    parseFloat(style.paddingBottom)
  );
};

/**
 * The caption templates, each a small moving picture the compositor drew of
 * a few words, in rows of three as the scene layouts are. Two rows show at
 * once and the rest scroll, so the panel keeps its height however many
 * templates there are; it opens with the chosen one in view. Each shows its
 * saved look with the layout options set by hand, as choosing it would set
 * the captions, and the chosen one shows the captions as they are set now.
 * A press takes the template's style up whole, but for the layout set by
 * hand; a right click, or the menu key, offers to put the template back to
 * how it ships.
 */
export function CaptionTemplateTiles({
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
  const shown = useMemo(
    () =>
      templates.styles.map((template) =>
        template.template === style.template
          ? style
          : withLayout(layout, template),
      ),
    [layout, style, templates.styles],
  );
  const strips = useCaptionPreviews(shown);
  const frameRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);

  // The two rows' height is read off the tiles and set on the frame as they
  // resize, so the panel never draws twice for it.
  useLayoutEffect(() => {
    const frame = frameRef.current;
    const grid = gridRef.current;
    if (!frame || !grid) return;
    const fit = () => {
      frame.style.maxHeight = `${String(rowsHeight(grid, ROWS_SHOWN))}px`;
    };
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(grid);
    return () => {
      observer.disconnect();
    };
  }, []);

  /** Brings the chosen template into view as the panel opens, and leaves the
   * tiles where they are after, so a choice never moves one under the
   * pointer. */
  const showChosen = (viewport: HTMLElement) => {
    const chosen =
      gridRef.current?.querySelector<HTMLElement>("[data-selected]");
    if (!chosen) return;
    const offset =
      chosen.getBoundingClientRect().bottom -
      viewport.getBoundingClientRect().bottom;
    if (offset > 0) viewport.scrollTop += offset;
  };

  /** The template of the tile `target` sits in, by its place among them. */
  const tileOf = (target: EventTarget) => {
    const tile = (target as HTMLElement).closest("button");
    if (!tile) return null;
    const at = [...(gridRef.current?.children ?? [])].indexOf(tile);
    return at < 0 ? null : { template: shown[at].template, tile };
  };

  return (
    <div
      onContextMenu={(event) => {
        const found = tileOf(event.target);
        if (!found) return;
        event.preventDefault();
        if (!isDisabled)
          void openMenu(
            found.template,
            pointerAnchor(event.clientX, event.clientY),
          );
      }}
      onKeyDown={(event) => {
        const isMenuKey =
          event.key === "ContextMenu" ||
          (event.key === "F10" && event.shiftKey);
        const found = isMenuKey ? tileOf(event.target) : null;
        if (!found || isDisabled) return;
        event.preventDefault();
        void openMenu(
          found.template,
          boundsAnchor(found.tile.getBoundingClientRect()),
        );
      }}
      ref={frameRef}
    >
      <ScrollArea
        constrainHeight
        onInitialized={showChosen}
        rootClassName="max-h-[inherit]"
      >
        <ToggleButtonGroup
          aria-label={t("editor-panels-captions-template")}
          // Room for a tile's focus ring at the edges it scrolls past.
          className="grid justify-between gap-y-control p-focus-safe"
          disallowEmptySelection
          isDisabled={isDisabled}
          onSelectionChange={(keys) => {
            const chosen = templates.styles.find((template) =>
              keys.has(template.template),
            );
            if (chosen) onChoose(chosen);
          }}
          ref={gridRef}
          selectedKeys={new Set([style.template])}
          selectionMode="single"
          // Each column a whole number of pixels, so every tile starts on one.
          style={{
            gridTemplateColumns:
              "repeat(3, round(down, calc((100% - 2 * var(--spacing-control)) / 3), 1px))",
          }}
        >
          {shown.map((template, index) => (
            <ToggleButton
              aria-label={captionTemplateLabel(template.template)}
              className={cn(
                "flex cursor-default flex-col items-stretch gap-control-inset rounded-control p-control-inset transition-colors",
                focusStyles,
                elementFocusVisible,
                style.template === template.template &&
                  "bg-primary-surface text-primary-fg",
                isDisabled && "opacity-50",
              )}
              id={template.template}
              key={template.template}
            >
              <CaptionPreviewStripView strip={strips?.[index]} />
              <Text className="text-center text-inherit" variant="footnote">
                {templates.defaultTemplate === template.template
                  ? t("editor-panels-captions-template-default", {
                      name: captionTemplateLabel(template.template),
                    })
                  : captionTemplateLabel(template.template)}
              </Text>
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </ScrollArea>
    </div>
  );
}
