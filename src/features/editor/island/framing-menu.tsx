// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown } from "lucide-react";
import { useListFormatter, useNumberFormatter } from "react-aria";

import { t } from "../../../i18n/i18n";
import { boundsAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";
import { listedFormats } from "../recording/formats/format-name";

import { IslandLineButton } from "./island-line-button";

import type { PopupPanelItem } from "../../popup-panel/store";
import type { ToolPanelFormats } from "../tools/tool-panel-frame";
import type { FramingPatch } from "../tools/tool-panel-patch";

// At the start of the Screen and Scene islands' row while a platform is in
// hand: which platforms the framing in hand is for. A platform framed its
// own way, or sharing its framing, says which platforms it applies to and
// ticks others of its canvas to share it with, or unticks one to give it
// back to the main canvas. One still following the main canvas says so,
// and can take up the framing of another platform of its canvas instead.
// The timeline's version lines mark where a platform is framed its own way.

const MENU = "framing:";
const MAIN = "main";
const MENU_WIDTH = 240;

export function FramingMenu({
  formats,
  onPatch,
}: {
  formats: ToolPanelFormats | undefined;
  onPatch: (patch: FramingPatch) => void;
}) {
  const list = useListFormatter({ type: "conjunction" });
  const count = useNumberFormatter();
  const active = formats?.list.find((shape) => shape.id === formats.activeId);
  const isApplied =
    active !== undefined && (active.ownWay || active.formats.length > 1);
  const open = usePopupMenu({
    idPrefix: MENU,
    label: t("editor-island-framing"),
    mode: isApplied ? "menu" : "select",
    onSelect: (id) => {
      if (!formats || !active) return;
      if (!isApplied) {
        if (id !== MAIN) onPatch({ kind: "follow", shape: id });
        return;
      }
      if (id === formats.formatId) return;
      onPatch({
        isShared: !active.formats.some((format) => format.id === id),
        kind: "share",
        platform: id,
      });
    },
    showsAllItems: true,
    width: MENU_WIDTH,
  });
  // The main canvas, the first shape, is what the platforms follow.
  if (!formats || !active || formats.list[0]?.id === active.id) return null;
  const named = listedFormats(formats);
  const nameOf = (id: string) =>
    named.find((each) => each.id === id)?.name ?? id;
  // Only platforms of the same canvas can share one framing.
  const alike = formats.list
    .slice(1)
    .filter(
      (shape) => shape.width === active.width && shape.height === active.height,
    );
  // The button stays short: what it does, and how many platforms share the
  // framing where it is more than this one. The platforms are named for
  // assistive technology, and in the menu.
  const sharing = active.formats.length;
  const description = isApplied
    ? t("editor-island-framing-applies", {
        platforms: list.format(
          active.formats.map((format) => nameOf(format.id)),
        ),
      })
    : t("editor-island-framing-follows-main");
  const menu = (): { items: PopupPanelItem[]; selectedIds: string[] } => {
    const current = t("editor-island-framing-current");
    if (isApplied) {
      const section = t("editor-island-framing-applies-heading");
      return {
        items: alike.flatMap((shape) =>
          shape.formats.map((format) => ({
            ...(format.id === formats.formatId ? { detail: current } : {}),
            id: format.id,
            label: nameOf(format.id),
            section,
            togglesInPlace: true,
          })),
        ),
        selectedIds: active.formats.map((format) => format.id),
      };
    }
    const section = t("editor-island-framing-follows-heading");
    return {
      items: [
        {
          detail: current,
          id: MAIN,
          label: t("editor-island-framing-main"),
          section,
        },
        ...alike
          .filter((shape) => shape.ownWay && shape !== active)
          .map((shape) => ({
            id: shape.id,
            label: list.format(
              shape.formats.map((format) => nameOf(format.id)),
            ),
            section,
          })),
      ],
      selectedIds: [MAIN],
    };
  };
  return (
    <IslandLineButton
      aria-haspopup="menu"
      aria-label={description}
      className="w-auto shrink-0"
      onPress={(event) => {
        void open({
          anchor: boundsAnchor(event.target.getBoundingClientRect()),
          ...menu(),
        });
      }}
    >
      <span aria-hidden className="whitespace-nowrap">
        {isApplied
          ? t("editor-island-framing-applies-heading")
          : t("editor-island-framing-follows-heading")}
      </span>
      {isApplied && sharing > 1 ? (
        <span aria-hidden className="text-fg-secondary tabular-nums">
          {count.format(sharing)}
        </span>
      ) : null}
      <ChevronDown
        aria-hidden
        className="size-icon-sm shrink-0 text-fg-secondary"
      />
    </IslandLineButton>
  );
}
