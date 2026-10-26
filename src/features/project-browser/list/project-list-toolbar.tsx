// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { Text } from "../../../primitives/text/text";
import { formatBytes } from "../../editor/duration";

import type { KindFilter } from "./project-list";
import type { ProjectKind, ProjectSummary } from "../types";

const kindItems = (): { id: KindFilter; label: string }[] => [
  { id: "all", label: t("project-browser-kind-all") },
  { id: "screen", label: t("project-browser-kind-screen") },
  { id: "camera", label: t("project-browser-kind-camera") },
  { id: "audio", label: t("project-browser-kind-audio") },
  { id: "screenshot", label: t("project-browser-kind-screenshot") },
];

export type ProjectListToolbarProps = {
  kind: KindFilter;
  /** Kinds among the source's projects; the others cannot be picked. */
  kindsPresent: ReadonlySet<ProjectKind>;
  /** What the list shows. */
  listed: readonly ProjectSummary[];
  onKindChange: (kind: KindFilter) => void;
  /** Sends everything in Recently Deleted on to the Trash; only there. */
  onEmpty?: () => void;
};

/**
 * Above the projects: the kind filter, then at the trailing edge how many
 * are listed and the disk they take, and in Recently Deleted the way to empty
 * it. What can be done to a choice is on the island, not here.
 */
export function ProjectListToolbar({
  kind,
  kindsPresent,
  listed,
  onEmpty,
  onKindChange,
}: ProjectListToolbarProps) {
  const bytes = listed.reduce(
    (total, { sizeBytes }) => total + (sizeBytes ?? 0),
    0,
  );
  const kinds = kindItems();

  return (
    // Wraps in a narrow window rather than running off it; the summary keeps
    // to the trailing edge on whichever line it lands.
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
      <div className="ms-auto flex shrink-0 items-center gap-separate">
        <Text className="tabular-nums" variant="caption">
          {bytes > 0
            ? t("project-browser-summary", {
                count: listed.length,
                size: formatBytes(bytes),
              })
            : t("project-browser-project-count", { count: listed.length })}
        </Text>
        {onEmpty ? (
          <ConfirmActionButton
            confirmLabel={t("project-browser-empty-recently-deleted-confirm")}
            isDisabled={listed.length === 0}
            label={t("project-browser-empty-recently-deleted")}
            onConfirm={onEmpty}
          />
        ) : null}
      </div>
    </div>
  );
}
