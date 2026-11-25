// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useRef, useState } from "react";

import { formatsOfShape } from "../recording/formats/recording-formats";

import { withSizeScale } from "./formats/screenshot-format-resolve";
import { screenshotSaveJobs } from "./formats/screenshot-save-jobs";
import { publishedWorkspace } from "./formats/screenshot-show-in";
import { useScreenshotFormats } from "./formats/use-screenshot-formats";
import { newestScreenshotLayer } from "./screenshot-blank";
import { defaultScreenshotOutput } from "./screenshot-output";
import {
  reopenedScreenshotWorkspace,
  seedScreenshotWorkspace,
} from "./screenshot-seed";
import { useScreenshotLayers } from "./use-screenshot-layers";
import { useScreenshotProject } from "./use-screenshot-project";

import type { ScreenshotWorkspaceOutputSettings } from "./screenshot-output";
import type { ScreenshotFormats } from "../../../bindings/ScreenshotFormats";
import type { EditorArtifact } from "../types";

/**
 * The open screenshot's workspace, the layer chosen on it, and the formats
 * it is made in: seeded as a screenshot opens, from its project where it
 * kept them and from the remembered look where not; saved with its project
 * as they change; and each capture taken since laid out as it arrives.
 * `seededArtifactId` is the screenshot the window has seeded, which nothing
 * is saved or laid out for before.
 */
export function useScreenshotWorkspace({
  artifact,
  persisted,
  seededArtifactId,
}: {
  artifact: EditorArtifact | null;
  persisted: Omit<Parameters<typeof seedScreenshotWorkspace>[0], "artifact">;
  seededArtifactId: number | undefined;
}) {
  const [workspace, setWorkspace] = useState<ScreenshotWorkspaceOutputSettings>(
    () => ({
      ...defaultScreenshotOutput(1, 1),
      items: [],
    }),
  );
  const [selectedItemId, setSelectedItemId] = useState<number | null>(null);
  const [formats, setFormats] = useState<ScreenshotFormats | null>(null);
  const seenItemIdsRef = useRef<Set<number>>(new Set());
  const artifactId = artifact?.id;
  useEffect(() => {
    const screenshot = artifact?.kind === "screenshot" ? artifact : null;
    /* eslint-disable @eslint-react/set-state-in-effect */
    setWorkspace(
      screenshot?.projectWorkspace
        ? reopenedScreenshotWorkspace(screenshot, screenshot.projectWorkspace)
        : seedScreenshotWorkspace({ artifact, ...persisted }),
    );
    setFormats(screenshot?.projectFormats ?? null);
    setSelectedItemId(
      screenshot
        ? (newestScreenshotLayer(
            screenshot.items.filter(
              (item) => !screenshot.deletedItemIds.includes(item.id),
            ),
          )?.id ?? null)
        : null,
    );
    /* eslint-enable @eslint-react/set-state-in-effect */
    seenItemIdsRef.current = new Set(
      screenshot?.items.map((item) => item.id) ?? [],
    );
    // Seeded once per screenshot, as the window's other settings are.
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [artifactId]);
  const seeded =
    artifact?.kind === "screenshot" && seededArtifactId === artifact.id
      ? artifact
      : null;
  useScreenshotProject(seeded?.id, workspace, formats);
  useScreenshotLayers({
    onSelectedItemChange: setSelectedItemId,
    screenshot: seeded,
    seenRef: seenItemIdsRef,
    setWorkspace,
    workspace,
  });
  const items = artifact?.kind === "screenshot" ? artifact.items : null;
  const blank = useMemo(
    () =>
      new Set(
        (items ?? []).filter((item) => item.isBlank).map((item) => item.id),
      ),
    [items],
  );
  const formatsControl = useScreenshotFormats({
    blank,
    formats,
    isScreenshot: artifact?.kind === "screenshot",
    onFormatsChange: setFormats,
    onWorkspaceChange: setWorkspace,
    workspace,
  });
  const leftOut = formatsControl?.snapshot.leftOut;
  const sizeScale = formatsControl?.snapshot.sizeScale ?? 1;
  // A layer the format in hand leaves out is not there to work on.
  const isChoiceLeftOut =
    selectedItemId !== null && (leftOut?.layers.has(selectedItemId) ?? false);
  const published = useMemo(
    () =>
      withSizeScale(
        leftOut && (leftOut.layers.size > 0 || leftOut.annotations.size > 0)
          ? publishedWorkspace(workspace, leftOut)
          : workspace,
        sizeScale,
      ),
    [leftOut, sizeScale, workspace],
  );
  const [savesAll, setSavesAll] = useState(true);
  const isMadeForMore = (formats?.list.flatMap(formatsOfShape).length ?? 0) > 1;
  return {
    /** Whether an export writes every format; null with one. */
    allFormats: isMadeForMore ? savesAll : null,
    formats,
    formatsControl,
    /** The format in hand as it is published, copied or saved alone:
     * without what it leaves out. */
    published,
    /** An export under `fileStem`: its name, and the files it writes in
     * place of the window's workspace as it is, every format or the one in
     * hand without what it leaves out; none where it writes that. */
    saveTarget: (fileStem: string) => ({
      fileStem,
      screenshots:
        (savesAll &&
          screenshotSaveJobs({
            fileStem,
            formats,
            inHand: { blank, workspace },
          })) ||
        (published === workspace ? null : [{ fileStem, workspace: published }]),
    }),
    selectedItemId: isChoiceLeftOut ? null : selectedItemId,
    setFormats,
    /** Sets `allFormats`; none outside a screenshot. */
    setSavesAll: artifact?.kind === "screenshot" ? setSavesAll : undefined,
    setSelectedItemId,
    setWorkspace,
    workspace,
  };
}
