// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { ProjectBrowser, type ProjectBrowserProps } from "./project-browser";
import { projectsFor } from "./project-browser-fixtures";

import type { ProjectSummary } from "./types";

/** The browser in a story, answering some events as the app would so it can
 * be tried: the sidebar switches source, and Delete takes the cards away
 * with an Undo on the island that brings them back. Each event still
 * reaches its arg, for the Actions panel. `updateArgs` is the story's own,
 * from `useArgs`, which only a story's render may call. */
export function LiveProjectBrowser({
  updateArgs,
  ...args
}: ProjectBrowserProps & {
  updateArgs: (next: Partial<ProjectBrowserProps>) => void;
}) {
  const removedRef = useRef<ProjectSummary[]>([]);
  return (
    <ProjectBrowser
      {...args}
      onDelete={(files) => {
        args.onDelete(files);
        const listed = args.projects ?? [];
        removedRef.current = listed.filter(({ file }) => files.includes(file));
        updateArgs({
          deleted: {
            count: files.length,
            title: removedRef.current[0]?.title ?? "",
          },
          projects: listed.filter(({ file }) => !files.includes(file)),
        });
      }}
      onSelect={(next) => {
        args.onSelect(next);
        updateArgs({ projects: projectsFor(next), selected: next });
      }}
      onUndoDelete={() => {
        args.onUndoDelete?.();
        updateArgs({
          deleted: null,
          projects: [...(args.projects ?? []), ...removedRef.current],
        });
        removedRef.current = [];
      }}
    />
  );
}
