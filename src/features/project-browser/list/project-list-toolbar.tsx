// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";

import type { KindFilter } from "./project-list";
import type { ProjectKind } from "../types";

const kindItems = (): { id: KindFilter; label: string }[] => [
  { id: "all", label: t("project-browser-kind-all") },
  { id: "screen", label: t("project-browser-kind-screen") },
  { id: "camera", label: t("project-browser-kind-camera") },
  { id: "audio", label: t("project-browser-kind-audio") },
  { id: "screenshot", label: t("project-browser-kind-screenshot") },
];

export type ProjectListToolbarProps = {
  /** Whether anything is listed, which Empty needs. */
  isEmpty: boolean;
  kind: KindFilter;
  /** Kinds among the source's projects; the others cannot be picked. */
  kindsPresent: ReadonlySet<ProjectKind>;
  onKindChange: (kind: KindFilter) => void;
  /** Sends everything in Recently Deleted on to the Trash; only there. */
  onEmpty?: () => void;
};

/**
 * Above the projects: the kind filter, and in Recently Deleted the way to
 * empty it at the trailing edge. How many are listed and what they take is
 * the storage strip's, and what can be done to a choice is on the island.
 */
export function ProjectListToolbar({
  isEmpty,
  kind,
  kindsPresent,
  onEmpty,
  onKindChange,
}: ProjectListToolbarProps) {
  const kinds = kindItems();

  return (
    // Wraps in a narrow window rather than running off it; Empty keeps to
    // the trailing edge on whichever line it lands.
    <div className="flex min-h-control-height flex-wrap items-center gap-separate pe-inset">
      <PillGroup
        aria-label={t("project-browser-kind")}
        disabledIds={kinds
          .filter(({ id }) => id !== "all" && !kindsPresent.has(id))
          .map(({ id }) => id)}
        items={kinds}
        onSelectionChange={(id) => {
          onKindChange(id as KindFilter);
        }}
        selected={kind}
      />
      {onEmpty ? (
        <ConfirmActionButton
          className="ms-auto"
          confirmLabel={t("project-browser-empty-recently-deleted-confirm")}
          isDisabled={isEmpty}
          label={t("project-browser-empty-recently-deleted")}
          onConfirm={onEmpty}
        />
      ) : null}
    </div>
  );
}
