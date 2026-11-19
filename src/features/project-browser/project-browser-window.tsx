// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { newScript } from "../teleprompter/api";

import { useProjectReadings } from "./activity/use-project-readings";
import {
  addProjectLocation,
  deleteProject,
  duplicateProjects,
  emptyRecentlyDeleted,
  forgetRecentProject,
  getProjectLocations,
  hideProjectBrowser,
  listProjects,
  moveProjects,
  openOtherProject,
  openProjectFile,
  openProjectLocation,
  PROJECT_STILL_EVENT,
  PROJECTS_CHANGED_EVENT,
  removeProjectLocation,
  renameProject,
  restoreProjects,
  revealProject,
  trashProjects,
} from "./api";
import { ProjectBrowser } from "./project-browser";
import {
  RECENT,
  sourceFor,
  type ProjectLocation,
  type ProjectSummary,
} from "./types";
import { opening, useOpenFailures } from "./use-open-failures";
import { useProjectCopy } from "./use-project-copy";
import { useProjectThumbnails } from "./use-project-thumbnails";
import { useScrubStrips } from "./use-scrub-strips";
import { useUndoDelete } from "./use-undo-delete";

const message = (cause: unknown) =>
  cause instanceof Error ? cause.message : String(cause);

/** The project browser, kept in step with what is on disk. A notes project
 * opens in place of the projects through `onOpenNotes`, by its manifest. */
export function ProjectBrowserWindow({
  onOpenNotes,
}: {
  onOpenNotes: (project: string) => void;
}) {
  const [locations, setLocations] = useState<ProjectLocation[]>([]);
  const [selected, setSelected] = useState(RECENT);
  const [projects, setProjects] = useState<ProjectSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  // Bumped to read everything again: after an action, a change Rust reports,
  // or a return to the window, since projects can change outside the app.
  const [generation, setGeneration] = useState(0);

  useEffect(() => {
    // A read for a selection the user has since left is dropped.
    let current = true;
    Promise.all([getProjectLocations(), listProjects(sourceFor(selected))])
      .then(([nextLocations, nextProjects]) => {
        if (!current) return;
        setLocations(nextLocations);
        // A location whose drive has gone cannot be shown, so the window
        // goes to the projects folder, or to Recent while that is missing
        // too.
        if (
          nextLocations.some(
            ({ available, path }) => path === selected && !available,
          )
        ) {
          const home = nextLocations.find(
            ({ available, isDefault }) => isDefault && available,
          );
          setProjects(null);
          setSelected(home?.path ?? RECENT);
          return;
        }
        setProjects(nextProjects);
      })
      .catch((cause: unknown) => {
        if (current) setError(message(cause));
      });
    return () => {
      current = false;
    };
  }, [generation, selected]);

  const loadThumbnailRef = useRef<(file: string) => void>(() => undefined);
  const forgetStripRef = useRef<(file: string) => void>(() => undefined);

  useEffect(() => {
    const reload = () => {
      setGeneration((value) => value + 1);
    };
    const subscriptions = [
      listen(PROJECTS_CHANGED_EVENT, reload),
      listen<string>(PROJECT_STILL_EVENT, ({ payload: file }) => {
        loadThumbnailRef.current(file);
        // The strip is composed after the still; either may be new.
        forgetStripRef.current(file);
      }),
      getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        if (focused) reload();
      }),
    ];
    return () => {
      for (const subscription of subscriptions) {
        void subscription.then((unlisten) => {
          unlisten();
        });
      }
    };
  }, []);

  const select = (next: string) => {
    // Pressing the row already chosen asks for nothing new; clearing the list
    // then would leave it empty, with no change of selection to refill it.
    if (next === selected) return;
    setProjects(null);
    setSelected(next);
  };
  const run = (action: Promise<unknown>) => {
    setError(null);
    action
      .then(() => {
        setGeneration((value) => value + 1);
      })
      .catch((cause: unknown) => {
        setError(message(cause));
      });
  };
  const undo = useUndoDelete();
  const copy = useProjectCopy(run);
  const scrub = useScrubStrips();
  forgetStripRef.current = scrub.forget;
  useOpenFailures(setError);
  const readings = useProjectReadings(projects);
  const thumbs = useProjectThumbnails(projects);
  loadThumbnailRef.current = thumbs.load;
  const titleOf = (file: string | undefined) =>
    projects?.find((project) => project.file === file)?.title ?? "";

  return (
    <ProjectBrowser
      copying={copy.copying}
      deleted={undo.deleted}
      error={error}
      locations={locations}
      onAddLocation={() => {
        run(
          addProjectLocation().then((added) => {
            if (added) select(added);
          }),
        );
      }}
      onClose={() => void hideProjectBrowser()}
      onDelete={(files) => {
        const title = titleOf(files[0]);
        run(
          Promise.all(files.map(deleteProject)).then((deleted) => {
            undo.record(deleted, title);
          }),
        );
      }}
      // One copy at a time: the notice follows one.
      onDuplicate={
        copy.copying
          ? undefined
          : (files) => {
              copy.start(
                {
                  count: files.length,
                  destination: null,
                  kind: "duplicate",
                  title: titleOf(files[0]),
                },
                duplicateProjects(files),
              );
            }
      }
      onEmpty={() => {
        run(emptyRecentlyDeleted());
      }}
      onForget={(file) => {
        run(forgetRecentProject(file));
      }}
      onMinimize={() => void getCurrentWindow().minimize()}
      onMove={
        copy.copying
          ? undefined
          : (files, location) => {
              copy.start(
                {
                  count: files.length,
                  destination:
                    locations.find(({ path }) => path === location)?.name ??
                    null,
                  kind: "move",
                  title: titleOf(files[0]),
                },
                moveProjects(files, location),
              );
            }
      }
      onNewScript={() => {
        run(newScript());
      }}
      onOpen={(file) => {
        if (projects?.find((one) => one.file === file)?.kind === "notes") {
          onOpenNotes(file);
          return;
        }
        run(opening(openProjectFile(file)));
      }}
      onOpenFile={() => {
        run(opening(openOtherProject()));
      }}
      onOpenLocation={(path) => {
        run(openProjectLocation(path));
      }}
      onRemoveLocation={(path) => {
        if (path === selected) select(RECENT);
        run(removeProjectLocation(path));
      }}
      onRename={(file, title) => {
        setError(null);
        return renameProject(file, title).then(
          (renamed) => {
            // The card is listed again under its new manifest; the still it
            // showed stays with it, so it is not drawn empty until the new
            // name's still has been asked for.
            thumbs.setThumbnails((current) => ({
              ...current,
              [renamed]: current[file] ?? null,
            }));
            setGeneration((value) => value + 1);
          },
          (cause: unknown) => {
            setError(message(cause));
            throw cause;
          },
        );
      }}
      onRequestScrubStrip={scrub.request}
      onRestore={(files) => {
        run(restoreProjects(files));
      }}
      onReveal={(file) => {
        run(revealProject(file));
      }}
      onSelect={select}
      onTrash={(files) => {
        run(trashProjects(files));
      }}
      onUndoDelete={() => {
        run(restoreProjects(undo.take()));
      }}
      projects={projects}
      readings={readings}
      scrubStrips={scrub.strips}
      selected={selected}
      thumbnails={thumbs.thumbnails}
    />
  );
}
