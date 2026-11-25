// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { appLocale, t } from "../../../../i18n/i18n";
import {
  type PopupMenuAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";

import { listedFormats, shapeName } from "./format-name";
import { useFrameFormats } from "./frame-formats-context";

import type { FormatSyncAction } from "./format-sync";
import type { FrameFormatsControl } from "./use-recording-formats";
import type { PopupPanelItem } from "../../../popup-panel/store";

// A scene, annotation, song or sound effect, or the caption track offers,
// in its right-click menu on the strip and on the picture, what sets it
// apart by format (`format-sync.ts`): whether it is custom in the shape in
// hand, ticked in place, then a tick for each format it shows in, set by
// a press or a drag across them without the menu closing.

const PREFIX = "format-sync:";
const CUSTOM = `${PREFIX}custom`;
const LAYOUT_ALL = `${PREFIX}layout-all`;
const SHOW = `${PREFIX}show:`;
const MENU_WIDTH = 260;

/** The menu items setting `id` apart by format, with those ticked; none
 * with one format. Show in, a tick for every format, scrolls on its own
 * under the commands (`scrollingSection`). */
export const formatSyncMenu = (
  control: FrameFormatsControl | null,
  id: string,
): {
  items: PopupPanelItem[];
  selectedIds: string[];
  scrollingSection?: string;
} => {
  const snapshot = control?.snapshot;
  const sync = snapshot?.syncOf?.(id);
  if (!snapshot || !sync) return { items: [], selectedIds: [] };
  const named = listedFormats(snapshot);
  const inHand = named.find((each) => each.id === snapshot.formatId);
  const section = t("editor-frame-formats");
  const showIn = t("editor-frame-format-show-in");
  const shown = sync.formats.filter((format) => format.isShown);
  // A scene is laid out per framing, shared by the platforms sharing it;
  // an annotation per proportion.
  const custom = inHand
    ? sync.byFraming
      ? new Intl.ListFormat(appLocale().formatLocale, {
          type: "conjunction",
        }).format(
          named
            .filter((each) => each.shape === inHand.shape)
            .map((each) => each.name),
        )
      : shapeName(inHand)
    : "";
  const items: PopupPanelItem[] = [
    ...(sync.custom === null || !inHand
      ? []
      : [
          {
            id: CUSTOM,
            label: t("editor-frame-format-custom", { shape: custom }),
            section,
            togglesInPlace: true,
          },
        ]),
    ...(sync.canSpread
      ? [
          {
            id: LAYOUT_ALL,
            label: t("editor-frame-format-layout-all"),
            section,
          },
        ]
      : []),
    ...sync.formats.map((format) => ({
      id: `${SHOW}${format.id}`,
      // Leaving it out of every format would be deleting it.
      keepsOneOn: true,
      label: named.find((each) => each.id === format.id)?.name ?? format.id,
      section: showIn,
      togglesInPlace: true,
    })),
  ];
  return {
    items,
    scrollingSection: showIn,
    selectedIds: [
      ...(sync.custom ? [CUSTOM] : []),
      ...shown.map((format) => `${SHOW}${format.id}`),
    ],
  };
};

/** What the format item `itemId` asks, where it is one. */
const actionOf = (itemId: string): FormatSyncAction | null => {
  if (itemId === CUSTOM) return { kind: "custom" };
  if (itemId === LAYOUT_ALL) return { kind: "layout-all" };
  if (itemId.startsWith(SHOW))
    return { format: itemId.slice(SHOW.length), kind: "show" };
  return null;
};

/** Takes the format item `itemId` for `id`, where it is one; whether it
 * was. */
export const pickedFormatSync = (
  control: FrameFormatsControl | null,
  itemId: string,
  id: string,
) => {
  if (!itemId.startsWith(PREFIX)) return false;
  const action = actionOf(itemId);
  if (action) control?.onPatch({ action, id, kind: "sync" });
  return true;
};

/**
 * A right press on a scene block, an annotation bar, a song or sound
 * effect, or the captions on the strip: opens a menu of what sets it apart
 * by format, and answers whether it did, so the strip's own menu opens
 * elsewhere.
 */
export function useFormatItemMenu() {
  const formats = useFrameFormats();
  const formatsRef = useRef(formats);
  formatsRef.current = formats;
  const open = usePopupMenu({
    idPrefix: "format-item:",
    label: t("editor-frame-formats"),
    mode: "menu",
    onSelect: (itemId, id) => {
      pickedFormatSync(formatsRef.current, itemId, id);
    },
    width: MENU_WIDTH,
  });
  return (anchor: PopupMenuAnchor, id: string) => {
    const menu = formatSyncMenu(formats, id);
    if (menu.items.length === 0) return false;
    void open({ anchor, context: id, ...menu });
    return true;
  };
}
