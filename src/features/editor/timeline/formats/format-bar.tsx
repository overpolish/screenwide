// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Plus } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { IconButton } from "../../../../primitives/button/icon-button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { revealChosen } from "../../../../primitives/scroll-area/reveal-chosen";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { TooltipTrigger } from "../../../../primitives/tooltip/tooltip-trigger";
import {
  boundsAnchor,
  pointerAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";
import { formatName } from "../../recording/formats/format-name";
import { useFrameFormats } from "../../recording/formats/frame-formats-context";
import {
  FRAME_SIZE_PRESETS,
  frameSizePresetName,
  frameSizePresetPlatform,
} from "../../tools/frame/frame-sizes";

import { FormatChipGlyph } from "./format-chip-glyph";
import { PhoneMenu } from "./phone-menu";

/** Brings the format in hand into view where the formats scroll, leaving
 * the row where it is while that format already shows whole. */
const showInHand = (viewport: HTMLElement) => {
  const chosen = viewport.querySelector("[data-selected]");
  if (!chosen) return;
  const box = viewport.getBoundingClientRect();
  const item = chosen.getBoundingClientRect();
  if (item.left < box.left || item.right > box.right)
    revealChosen(viewport, "[data-selected]", "horizontal");
};

const ADD_MENU = "format-add:";
const FORMAT_MENU = "format-actions:";
const MENU_WIDTH = 260;
const REMOVE = "remove";
/** The main canvas before a platform is added, when it has no id yet. */
const ONLY_FORMAT = "only";

/**
 * The recording's formats, at the end of the timeline's transport row:
 * the main canvas, the one worked in and sized freely, named by its size;
 * then every platform added, in one group whose glyphs show which way up
 * each is, scrolling sideways once the row runs out of room. A press puts
 * one in hand; a right press on a platform offers to remove it. Add offers
 * the platforms' templates not added yet; the phone preview and its phone
 * follow it (`PhoneMenu`), waiting unless a platform with a phone app is
 * in hand. Where formats differ is drawn under the strip's overview
 * (`minimap-versions.tsx`). Nothing shows outside a recording.
 */
export function FormatBar({ isLocked = false }: { isLocked?: boolean }) {
  const control = useFrameFormats();
  const addRef = useRef<HTMLButtonElement>(null);
  const openAdd = usePopupMenu({
    idPrefix: ADD_MENU,
    label: t("editor-frame-format-add"),
    mode: "menu",
    onSelect: (preset) => {
      control?.onPatch({ kind: "add", preset });
    },
    // The bar sits at the window's foot, with the room above it.
    opensAbove: true,
    width: MENU_WIDTH,
  });
  const openActions = usePopupMenu({
    idPrefix: FORMAT_MENU,
    label: t("editor-frame-formats"),
    mode: "menu",
    onSelect: (action, id) => {
      if (action === REMOVE) control?.onPatch({ id, kind: "remove" });
    },
    width: MENU_WIDTH,
  });
  // The platform put in hand, as one just added at the row's end, is
  // brought into view where the platforms scroll, and kept there as the
  // row narrows.
  const [chips, setChips] = useState<HTMLElement | null>(null);
  const inHandId = control?.snapshot.formatId ?? null;
  useLayoutEffect(() => {
    if (!chips) return;
    showInHand(chips);
    const observer = new ResizeObserver(() => {
      showInHand(chips);
    });
    observer.observe(chips);
    return () => {
      observer.disconnect();
    };
  }, [chips, inHandId]);
  if (!control) return null;
  const { snapshot } = control;
  const main = snapshot.list.length > 0 ? snapshot.list[0] : undefined;
  const platformShapes = snapshot.list.slice(1);
  const mainSize = main ?? snapshot;
  const mainId = main?.id ?? ONLY_FORMAT;
  const platforms = platformShapes.flatMap((shape) =>
    shape.formats.map((format) => {
      const label = formatName({ ...shape, preset: format.preset });
      return {
        icon: <FormatChipGlyph isLeavingOut={format.leavesOut} size={shape} />,
        id: format.id,
        label,
        ...(format.leavesOut
          ? {
              ariaLabel: t("editor-frame-format-leaves-out", { format: label }),
            }
          : {}),
      };
    }),
  );
  const isMainLeavingOut = main?.formats[0]?.leavesOut === true;
  const isMainInHand = !main || snapshot.activeId === main.id;
  // The template the main canvas is made for, which names it, and which is
  // not added again as a platform of its own.
  const mainPreset = main?.formats[0]?.preset ?? null;
  const made = new Set([
    ...platforms.map((format) => format.id),
    ...(mainPreset ? [mainPreset] : []),
  ]);
  const addable = (snapshot.templates ?? FRAME_SIZE_PRESETS).filter(
    (preset) => !made.has(preset.id),
  );
  return (
    <div
      className="flex min-w-0 items-center gap-inline"
      // Only a platform offers a menu; elsewhere a right press does nothing.
      onContextMenu={(event) => {
        event.preventDefault();
      }}
    >
      <PillGroup
        aria-label={t("editor-frame-format-main")}
        isDisabled={isLocked}
        items={[
          {
            ariaLabel: isMainLeavingOut
              ? t("editor-frame-format-leaves-out", {
                  format: t("editor-frame-format-main"),
                })
              : t("editor-frame-format-main"),
            icon: (
              <FormatChipGlyph
                isLeavingOut={isMainLeavingOut}
                size={mainSize}
              />
            ),
            id: mainId,
            label: formatName({ ...mainSize, preset: mainPreset }),
          },
        ]}
        onSelectionChange={(id) => {
          if (id !== ONLY_FORMAT) control.onPatch({ id, kind: "switch" });
        }}
        selected={isMainInHand ? mainId : ""}
      />
      {platforms.length > 0 ? (
        <div
          className="flex min-w-0"
          onContextMenu={(event) => {
            const segment = (event.target as Element).closest("button");
            const segments = [
              ...event.currentTarget.querySelectorAll("button"),
            ];
            const format = segment
              ? platforms[segments.indexOf(segment)]
              : undefined;
            if (!format) return;
            void openActions({
              anchor: pointerAnchor(event.clientX, event.clientY),
              context: format.id,
              items: [
                {
                  id: REMOVE,
                  label: t("editor-frame-format-remove-named", {
                    format: format.label,
                  }),
                },
              ],
            });
          }}
        >
          {/* The platforms are what grows, so they scroll sideways once the
              row runs out of room; the controls round them keep their
              size. */}
          <ScrollArea
            onInitialized={setChips}
            orientation="horizontal"
            rootClassName="min-w-0 rounded-control"
            scrollbarHidden
          >
            <PillGroup
              aria-label={t("editor-frame-format-platforms")}
              className="w-max"
              isDisabled={isLocked}
              items={platforms}
              onSelectionChange={(id) => {
                control.onPatch({ id, kind: "switch" });
              }}
              selected={isMainInHand ? "" : (snapshot.formatId ?? "")}
            />
          </ScrollArea>
        </div>
      ) : null}
      <TooltipTrigger tooltip={{ label: t("editor-frame-format-add") }}>
        <IconButton
          aria-label={t("editor-frame-format-add")}
          isDisabled={isLocked || addable.length === 0}
          onPress={() => {
            const bounds = addRef.current?.getBoundingClientRect();
            if (!bounds) return;
            void openAdd({
              anchor: boundsAnchor(bounds),
              items: addable.map((preset) => ({
                id: preset.id,
                label: frameSizePresetName(preset.id),
                section: frameSizePresetPlatform(preset.id),
              })),
            });
          }}
          ref={addRef}
        >
          <Plus aria-hidden />
        </IconButton>
      </TooltipTrigger>
      <PhoneMenu
        formats={snapshot}
        // Only a platform watched in a phone app (Shorts, TikTok, Reels)
        // has an app to preview; LinkedIn, X, the wide platforms and the
        // main canvas have none.
        isDisabled={isLocked || snapshot.overlay === null}
        onPatch={control.onPatch}
      />
    </div>
  );
}
