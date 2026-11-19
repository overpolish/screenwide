// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { ImageIcon, Monitor, Video } from "lucide-react";
import { useMemo, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Alert } from "../../../../primitives/alert/alert";
import { SearchField } from "../../../../primitives/input-fields/search-field";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { Text } from "../../../../primitives/text/text";
import { formatDuration } from "../../../editor/duration";

import type { ProjectSummary } from "../../../project-browser/types";

/** The recordings and screenshots a cue can show, found by name, each with
 * its still. */
export function CueSources({
  hasFailed,
  isMaking,
  onChoose,
  projects,
  stills,
}: {
  hasFailed: boolean;
  isMaking: boolean;
  onChoose: (project: ProjectSummary) => void;
  /** Newest first; null until listed. */
  projects: readonly ProjectSummary[] | null;
  /** Each still's address, by its project's manifest. */
  stills: Readonly<Record<string, string>>;
}) {
  const [query, setQuery] = useState("");
  const shown = useMemo(() => {
    const words = query.trim().toLocaleLowerCase();
    return (projects ?? []).filter((project) =>
      project.title.toLocaleLowerCase().includes(words),
    );
  }, [projects, query]);

  return (
    <div className="flex size-full flex-col gap-inline p-inset-sm">
      <SearchField
        aria-label={t("teleprompter-cue-search")}
        autoFocus
        onChange={setQuery}
        placeholder={t("teleprompter-cue-search")}
        value={query}
      />
      {hasFailed ? (
        <Alert color="error">{t("teleprompter-cue-failed")}</Alert>
      ) : null}
      <ScrollArea rootClassName="min-h-0 grow">
        {projects && shown.length === 0 ? (
          <Text
            className="p-inset text-center text-fg-secondary"
            variant="body"
          >
            {t("teleprompter-cue-none")}
          </Text>
        ) : (
          <ListBox
            aria-label={t("teleprompter-cue-sources")}
            className="w-full p-0"
            // Stills arrive one by one after the list; the items are drawn
            // again as each does.
            dependencies={[stills]}
            disabledKeys={isMaking ? shown.map((project) => project.file) : []}
            items={shown}
            onAction={(file) => {
              const project = shown.find((each) => each.file === file);
              if (project) onChoose(project);
            }}
          >
            {(project) => {
              const Icon =
                project.kind === "screenshot"
                  ? ImageIcon
                  : project.kind === "camera"
                    ? Video
                    : Monitor;
              const still = stills[project.file];
              return (
                <ListBoxItem
                  className="h-auto py-inset-sm"
                  id={project.file}
                  showsSelection={false}
                  textValue={project.title}
                >
                  <span className="flex items-center gap-separate">
                    <span
                      className={cn(
                        "grid h-9 w-16 shrink-0 place-items-center overflow-hidden rounded-sm",
                        !still && "bg-fill",
                      )}
                    >
                      {still ? (
                        <img
                          alt=""
                          className="size-full object-contain"
                          draggable={false}
                          src={still}
                        />
                      ) : (
                        <Icon aria-hidden className="size-icon-sm" />
                      )}
                    </span>
                    <span className="flex min-w-0 flex-col">
                      <span className="truncate" dir="auto">
                        {project.title}
                      </span>
                      {/* A screenshot is made a cue as it is picked, a
                          recording opens on its moments; this says which. */}
                      <span className="text-caption text-fg-secondary">
                        {project.kind === "screenshot"
                          ? t("teleprompter-cue-screenshot")
                          : formatDuration(project.durationMs ?? 0)}
                      </span>
                    </span>
                  </span>
                </ListBoxItem>
              );
            }}
          </ListBox>
        )}
      </ScrollArea>
    </div>
  );
}
