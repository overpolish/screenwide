// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import { EditorArtifact } from "../types";

import { newestScreenshotLayer } from "./screenshot-blank";
import { ScreenshotWorkspaceOutputSettings } from "./screenshot-output";
import { seedScreenshotItemOutput } from "./screenshot-seed";

type ScreenshotArtifact = Extract<EditorArtifact, { kind: "screenshot" }>;

/**
 * The layers a screenshot gains while it is open, laid out as they arrive:
 * a capture joining it is placed at its real size and taken in hand, the
 * newest picture being what a tool takes up. A workspace left with no layer,
 * as deleting its last picture leaves it, is given its blank layer back, or
 * a new one the size of the canvas, so the canvas always has a layer to draw
 * on (#41). `seenRef` holds the layers already laid out, which seeding the
 * screenshot resets.
 */
export function useScreenshotLayers({
  onSelectedItemChange,
  screenshot,
  seenRef,
  setWorkspace,
  workspace,
}: {
  onSelectedItemChange: (id: number | null) => void;
  /** The open screenshot, once the window has seeded it. */
  screenshot: ScreenshotArtifact | null;
  seenRef: React.RefObject<Set<number>>;
  setWorkspace: React.Dispatch<
    React.SetStateAction<ScreenshotWorkspaceOutputSettings>
  >;
  workspace: ScreenshotWorkspaceOutputSettings;
}) {
  const workspaceRef = useRef(workspace);
  workspaceRef.current = workspace;
  const items = screenshot?.items ?? null;
  useEffect(() => {
    if (!items) return;
    const added = items.filter((item) => !seenRef.current.has(item.id));
    seenRef.current = new Set(items.map((item) => item.id));
    const laidOut = new Set(workspaceRef.current.items.map((item) => item.id));
    onSelectedItemChange(
      newestScreenshotLayer(
        items.filter((item) => laidOut.has(item.id) || added.includes(item)),
      )?.id ?? null,
    );
    if (added.length === 0) return;
    setWorkspace((current) => ({
      ...current,
      items: [
        ...current.items,
        ...added.map((item) => ({
          id: item.id,
          // Placed at its real size: fitting is the user's to do.
          output: seedScreenshotItemOutput(current, item, { fit: false }),
        })),
      ],
    }));
  }, [items, onSelectedItemChange, seenRef, setWorkspace]);

  // Asked for once per screenshot: the blank layer asked for arrives as a
  // layer added above.
  const askedRef = useRef<number | null>(null);
  const isEmpty = workspace.items.length === 0;
  useEffect(() => {
    if (!screenshot || !isEmpty) return;
    const blank = screenshot.items.find((item) => item.isBlank);
    if (blank) {
      setWorkspace((current) =>
        current.items.length > 0
          ? current
          : {
              ...current,
              items: [
                {
                  id: blank.id,
                  output: seedScreenshotItemOutput(current, blank, {
                    fit: false,
                  }),
                },
              ],
            },
      );
      return;
    }
    if (askedRef.current === screenshot.id) return;
    askedRef.current = screenshot.id;
    invoke<null>("add_blank_screenshot_layer", {
      artifactId: screenshot.id,
      height: Math.round(workspaceRef.current.height),
      width: Math.round(workspaceRef.current.width),
    }).catch((cause: unknown) => {
      console.error("Could not give the canvas a blank layer", cause);
    });
  }, [screenshot, isEmpty, setWorkspace]);
}
