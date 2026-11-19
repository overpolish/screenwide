// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRef } from "react";

import { t } from "../../../i18n/i18n";
import {
  openPopupPanels,
  panelStartX,
  showPopupPanel,
} from "../../popup-panel/api";
import {
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "../../popup-panel/store";
import {
  type PopupMenuAnchor,
  pointerAnchor,
  usePopupMenu,
} from "../../popup-panel/use-popup-menu";

import { PICKER_SIZE } from "./picker/use-picker-size";

import type { ShownTeleprompterCue } from "./cue-api";

const MENU_WIDTH = 180;
const ID_PREFIX = "teleprompter-cue:";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

/** How often, and how many times, the menu's panel is asked whether it has
 * gone, in milliseconds and tries. */
const CLOSE_POLL_MS = 25;
const CLOSE_POLLS = 20;

const pause = (ms: number) =>
  new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });

/**
 * Resolves once the menu a choice was made in has left the shared panel.
 * The picker opens in that same panel, and the menu's going is told to
 * every window as it lands, which would put away a picker opened before
 * it: so the picker waits until the app has the panel closed, and a moment
 * more for that word to have reached this window.
 */
async function afterMenuCloses() {
  for (let poll = 0; poll < CLOSE_POLLS; poll++) {
    if (!(await openPopupPanels()).includes(SHARED_POPUP_PANEL)) break;
    await pause(CLOSE_POLL_MS);
  }
  await pause(CLOSE_POLL_MS * 2);
}

/**
 * The menus that add, replace and take out cues: one on a word of the
 * script, one on a cue's mark. Adding or replacing opens the picker in the
 * shared panel, under where the menu was opened; a word holds one cue, so
 * the cue made replaces any it had.
 */
export function useCueMenus({
  cues,
  onRemove,
  script,
}: {
  cues: readonly ShownTeleprompterCue[];
  onRemove: (id: string) => void;
  /** The script project's manifest. */
  script: string;
}) {
  const anchorRef = useRef<PopupMenuAnchor>(pointerAnchor(0, 0));

  const openPicker = async (at: number) => {
    const anchor = anchorRef.current;
    const id = `${ID_PREFIX}picker:${String(at)}`;
    usePopupPanelStore.getState().open(SHARED_POPUP_PANEL, {
      content: { at, kind: "cuePicker", script },
      focusContents: true,
      id,
      label: t("teleprompter-cue-picker"),
    });
    await showPopupPanel({
      anchor,
      focusContents: true,
      offset: new LogicalPosition(
        panelStartX(anchor, PICKER_SIZE.width),
        anchor.y + anchor.height + 4,
      ),
      parentWindowLabel: getCurrentWindow().label,
      size: new LogicalSize(PICKER_SIZE.width, PICKER_SIZE.height),
      triggerId: id,
    });
  };

  // The context names the word's start and, for a word with a cue, the
  // cue: `at` or `at:id`.
  const openMenu = usePopupMenu({
    idPrefix: ID_PREFIX,
    label: t("teleprompter-cue-menu"),
    mode: "menu",
    onSelect: (item, context) => {
      const [at, cue] = context.split(":");
      if (item === "remove") {
        onRemove(cue);
        return;
      }
      afterMenuCloses()
        .then(() => openPicker(Number(at)))
        .catch(report("Could not open the cue picker"));
    },
    width: MENU_WIDTH,
  });

  const menu = (anchor: PopupMenuAnchor, at: number) => {
    anchorRef.current = anchor;
    const cue = cues.find((each) => each.at === at);
    openMenu({
      anchor,
      context: cue ? `${String(at)}:${cue.id}` : String(at),
      items: cue
        ? [
            { id: "replace", label: t("teleprompter-cue-replace") },
            { id: "remove", label: t("teleprompter-cue-remove") },
          ]
        : [{ id: "add", label: t("teleprompter-cue-add") }],
    }).catch(report("Could not open the cue menu"));
  };

  return menu;
}
