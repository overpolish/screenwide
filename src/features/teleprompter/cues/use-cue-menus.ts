// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect, useRef } from "react";

import { t } from "../../../i18n/i18n";
import {
  closePopupMenu,
  type PopupMenuAnchor,
  pointerAnchor,
  usePopupMenu,
} from "../../popup-panel/use-popup-menu";

import { afterMenuCloses, openCuePanel } from "./cue-panel";

import type { ShownTeleprompterCue } from "./cue-api";

const MENU_WIDTH = 180;
const ID_PREFIX = "teleprompter-cue:";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

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

  // A menu or picker stands where its word was; scrolled, the word moves
  // from under it, so it goes.
  useEffect(() => {
    const close = () => {
      closePopupMenu(ID_PREFIX).catch(report("Could not close the cue menu"));
    };
    window.addEventListener("wheel", close, { passive: true });
    return () => {
      window.removeEventListener("wheel", close);
    };
  }, []);

  const openPicker = (at: number) =>
    openCuePanel({
      anchor: anchorRef.current,
      content: { at, kind: "cuePicker", script },
      id: `${ID_PREFIX}picker:${String(at)}`,
      label: t("teleprompter-cue-picker"),
    });

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
