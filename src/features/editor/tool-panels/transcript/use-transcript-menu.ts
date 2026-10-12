// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { PopupPanelItem } from "../../../popup-panel/store";
import {
  pointerAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";

import { ClientPoint } from "./transcript-edit-span";
import { TranscriptSceneMenu } from "./transcript-scenes";
import { TranscriptUnit, WordSelection } from "./transcript-units";

import type { TranscriptMark } from "./use-transcript-marks";

const MENU_PREFIX = "transcript-words:";
const SCENE_PREFIX = "scene:";
const MENU_WIDTH = 200;

export type TranscriptMenuAction =
  | "delete"
  | "edit"
  | "not-profanity"
  | "remove-kind"
  | "remove-same"
  | "restore"
  | "spelling";

/** What the menu needs of the words it opens on beyond the units: the mark
 * the first chosen word carries, and the marked run's text as shown. */
export type TranscriptMenuMark = { mark: TranscriptMark; text: string };

/** What a filler or a profane word adds to the menu, under a heading of its
 * kind, as the pin's actions head a section of their own: removing every one
 * written the same way, every one of its kind, and for profanity saying it
 * is not. */
const markItems = (marked: TranscriptMenuMark | null): PopupPanelItem[] => {
  if (!marked) return [];
  const isFiller = marked.mark === "filler";
  const items: PopupPanelItem[] = [
    {
      icon: "trash",
      id: "remove-same",
      label: t("editor-panels-transcript-remove-same", { text: marked.text }),
    },
    {
      icon: "trash",
      id: "remove-kind",
      label: isFiller
        ? t("editor-panels-transcript-remove-fillers")
        : t("editor-panels-transcript-remove-profanity"),
    },
    ...(isFiller
      ? []
      : [
          {
            icon: "allow" as const,
            id: "not-profanity",
            label: t("editor-panels-transcript-not-profanity"),
          },
        ]),
  ];
  const section = isFiller
    ? t("editor-panels-transcript-fillers")
    : t("editor-panels-transcript-profanity");
  return items.map((item) => ({ ...item, section }));
};

/** What the menu offers for `chosen`: editing where it has a word - a pause
 * is never edited - each way back only where some of the words need it, and
 * the mark's own items where the first word is marked. */
const items = (
  chosen: readonly TranscriptUnit[],
  marked: TranscriptMenuMark | null,
): PopupPanelItem[] => [
  ...(chosen.some((unit) => unit.kind === "word")
    ? [
        {
          icon: "edit" as const,
          id: "edit",
          label: t("editor-panels-transcript-edit"),
          shortcut: "Enter",
        },
      ]
    : []),
  ...(chosen.some((unit) => !unit.removed)
    ? [
        {
          icon: "trash" as const,
          id: "delete",
          label: t("editor-panels-delete"),
          shortcut: "Backspace",
        },
      ]
    : []),
  ...(chosen.some((unit) => unit.removed)
    ? [
        {
          icon: "restore" as const,
          id: "restore",
          label: t("editor-panels-transcript-restore"),
          shortcut: "Shift+Backspace",
        },
      ]
    : []),
  ...(chosen.some((unit) => unit.corrected)
    ? [
        {
          icon: "spelling" as const,
          id: "spelling",
          label: t("editor-panels-transcript-restore-spelling"),
        },
      ]
    : []),
  ...markItems(marked),
];

/** The scenes the words can be given, under a heading of their own, as the
 * mark's items are: each layout by its name. */
const sceneItems = (scenes: TranscriptSceneMenu | null): PopupPanelItem[] => {
  if (!scenes) return [];
  const section =
    scenes.type === "add"
      ? t("editor-panels-transcript-add-scene")
      : t("editor-panels-transcript-change-scene");
  return scenes.options.map((option) => ({
    icon: "scene",
    id: `${SCENE_PREFIX}${option.id}`,
    label: option.label,
    section,
  }));
};

/**
 * A right click on the transcript's words, answered with the app's own menu.
 * The words it acts on travel with it, so a pick reaches the ones it was
 * opened on; a scene picked reaches `onScene` with the option's id.
 */
export function useTranscriptMenu(
  on: Record<TranscriptMenuAction, (span: WordSelection) => void>,
  onScene: (span: WordSelection, optionId: string) => void,
) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: t("editor-panels-transcript-actions"),
    mode: "menu",
    onSelect: (itemId, context) => {
      const [first, last] = context.split("-").map(Number);
      if (itemId.startsWith(SCENE_PREFIX))
        onScene({ first, last }, itemId.slice(SCENE_PREFIX.length));
      else if (itemId in on)
        on[itemId as TranscriptMenuAction]({ first, last });
    },
    showsAllItems: true,
    width: MENU_WIDTH,
  });

  return ({
    marked,
    point,
    scenes,
    span,
    units,
  }: {
    marked: TranscriptMenuMark | null;
    point: ClientPoint;
    scenes: TranscriptSceneMenu | null;
    span: WordSelection;
    units: readonly TranscriptUnit[];
  }) =>
    openMenu({
      anchor: pointerAnchor(point.x, point.y),
      context: `${span.first.toString()}-${span.last.toString()}`,
      items: [
        ...items(units.slice(span.first, span.last + 1), marked),
        ...sceneItems(scenes),
      ],
    });
}
