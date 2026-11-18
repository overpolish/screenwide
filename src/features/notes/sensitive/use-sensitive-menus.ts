// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import {
  boundsAnchor,
  pointerAnchor,
  usePopupMenu,
} from "../../popup-panel/use-popup-menu";
import { type NotesDocument, removeTurns } from "../document/notes-document";

import {
  markNotPrivate,
  redactInTurn,
  redactThroughout,
  type SensitiveGroup,
  sensitiveKindLabel,
} from "./sensitive-edits";

import type { SensitiveLookup } from "./use-sensitive";
import type { SensitiveFind } from "../../../bindings/SensitiveFind";

const BADGE_PREFIX = "notes-sensitive-badge:";
const WORD_PREFIX = "notes-sensitive-word:";
const MENU_WIDTH = 240;

const REDACT = "redact";
const REDACT_THROUGHOUT = "redact-throughout";
const NOT_PRIVATE = "not-private";
const DELETE = "delete";

/** The menus that act on what may be private: a badge's, over what it
 * names in its paragraph, and a marked word's, opened by a right click on
 * it. Each action is one change to the notes, so one Undo takes it back:
 * redacting, deleting the paragraph, or marking the words as not private,
 * which stops them being pointed out anywhere in the notes. */
export function useSensitiveMenus({
  document,
  onChange,
  sensitive,
}: {
  document: NotesDocument;
  onChange: (document: NotesDocument) => void;
  sensitive: SensitiveLookup;
}) {
  const textOf = (turn: number) =>
    (document.turns[turn] as NotesDocument["turns"][number] | undefined)
      ?.text ?? "";
  const notPrivateItem = (section: string) => ({
    id: NOT_PRIVATE,
    label: t("notes-sensitive-not-private"),
    section,
  });
  const deleteItem = (section: string) => ({
    id: DELETE,
    label: t("notes-sensitive-delete-paragraph"),
    section,
  });
  const deleteTurn = (turn: number) => {
    onChange(removeTurns(document, turn, turn + 1));
  };
  const openBadgeMenu = usePopupMenu({
    idPrefix: BADGE_PREFIX,
    label: t("notes-sensitive-menu"),
    mode: "menu",
    onSelect: (id, context) => {
      const [turnText, kind] = context.split(":");
      const turn = Number(turnText);
      if (id === DELETE) {
        deleteTurn(turn);
        return;
      }
      const text = textOf(turn);
      const finds = sensitive(turn, text).filter((find) => find.kind === kind);
      if (finds.length === 0) return;
      if (id === REDACT) onChange(redactInTurn(document, turn, finds));
      if (id === NOT_PRIVATE)
        onChange(
          markNotPrivate(
            document,
            finds.map(({ end, start }) => text.slice(start, end)),
          ),
        );
    },
    width: MENU_WIDTH,
  });
  const openWordMenu = usePopupMenu({
    idPrefix: WORD_PREFIX,
    label: t("notes-sensitive-menu"),
    mode: "menu",
    onSelect: (id, context) => {
      const [turn, start, end] = context.split(":").map(Number);
      if (id === DELETE) {
        deleteTurn(turn);
        return;
      }
      const text = textOf(turn);
      if (end > text.length) return;
      if (id === REDACT)
        onChange(redactInTurn(document, turn, [{ end, start }]));
      if (id === REDACT_THROUGHOUT)
        onChange(redactThroughout(document, text.slice(start, end)));
      if (id === NOT_PRIVATE)
        onChange(markNotPrivate(document, [text.slice(start, end)]));
    },
    width: MENU_WIDTH,
  });
  return {
    /** Opens what can be done about `group` in turn `turn`, under the badge
     * at `bounds`. */
    openBadge: (turn: number, group: SensitiveGroup, bounds: DOMRect) => {
      const section = sensitiveKindLabel(group);
      void openBadgeMenu({
        anchor: boundsAnchor(bounds),
        context: `${String(turn)}:${group.kind}`,
        items: [
          {
            id: REDACT,
            label: t("notes-sensitive-redact-found", {
              count: group.finds.length,
            }),
            section,
          },
          notPrivateItem(section),
          deleteItem(section),
        ],
      });
    },
    /** Opens what can be done about `find` in turn `turn`, right clicked
     * at `point`. */
    openWord: (
      turn: number,
      find: SensitiveFind,
      point: { x: number; y: number },
    ) => {
      const section = sensitiveKindLabel({ finds: [find], kind: find.kind });
      void openWordMenu({
        anchor: pointerAnchor(point.x, point.y),
        context: `${String(turn)}:${String(find.start)}:${String(find.end)}`,
        items: [
          { id: REDACT, label: t("notes-sensitive-redact-here"), section },
          {
            id: REDACT_THROUGHOUT,
            label: t("notes-sensitive-redact-throughout"),
            section,
          },
          notPrivateItem(section),
          deleteItem(section),
        ],
      });
    },
  };
}
