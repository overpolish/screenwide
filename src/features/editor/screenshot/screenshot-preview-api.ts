// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { AnnotationStyle, ImageArt } from "../annotations/annotations";
import { type PreviewSelectionLayout } from "../api";
import { type ViewportCorners } from "../preview/preview-backdrop";

import {
  ScreenshotWorkspaceOutputSettings,
  normalizedScreenshotWorkspaceOutput,
} from "./screenshot-output";

import type { TextStyle } from "../../../bindings/TextStyle";
import type { AnnotationTool } from "../annotations/annotation-defaults";

// The screenshot workspace's native preview: started on an artifact, laid out
// as the page lays the viewport out, refreshed when its sources change, put
// to sleep while another window holds the editor, and stopped.

export const startScreenshotPreview = (artifactId: number, sessionId: number) =>
  invoke<null>("start_screenshot_preview", { artifactId, sessionId });

export const layoutScreenshotPreviewSurface = ({
  annotationCounterAngle,
  annotationDefaults,
  annotationImage,
  annotationTextLook,
  annotationTool,
  annotationZoom,
  interactionOutput,
  output,
  selectedAnnotationIds = [],
  selection,
  selectionTargets,
  ...layout
}: {
  backdrop: [number, number, number, number];
  interactionOutput: ScreenshotWorkspaceOutputSettings;
  nativeEditor: boolean;
  output: ScreenshotWorkspaceOutputSettings;
  panes: {
    index: number;
    rect: { height: number; width: number; x: number; y: number };
  }[];
  scale: number;
  /** Whether ⌘ (Ctrl) holds Select over the tool: the chosen layer's chrome
   * stands down and a press on it draws a band. */
  selectHeld: boolean;
  sessionId: number;
  viewport: { height: number; width: number; x: number; y: number };
  /** How round the viewport's corners are where it meets the chrome. */
  viewportCorners: ViewportCorners;
  /** Where a fresh counter's tail points, in radians clockwise from east:
   * whatever the last counter was turned to. */
  annotationCounterAngle?: number | null;
  /** The dress the next fresh arrow is drawn in: whatever the editor's last
   * annotation edit settled on. Absent until it has settled on anything, and
   * the native tool falls back to the accent at its own stroke. */
  annotationDefaults?: AnnotationStyle | null;
  /** The picture a fresh image shows: whatever the last image was given. */
  annotationImage?: ImageArt | null;
  /** The look a fresh text is dressed in: whatever the last text was. */
  annotationTextLook?: TextStyle | null;
  /** The annotation tool in hand. "select" hit-tests the annotations already on
   * the layer and lets every other press fall through to it; "arrow" and
   * "counter" also make a new annotation on empty picture. */
  annotationTool?: AnnotationTool;
  /** How many times a fresh lens enlarges: whatever the last lens was set
   * to. */
  annotationZoom?: number | null;
  /** Every annotation chosen on the selected layer. */
  selectedAnnotationIds?: readonly string[];
  selection?: PreviewSelectionLayout | null;
  selectionTargets?: PreviewSelectionLayout[] | null;
  /** The page's size, width then height, the viewport was measured in. */
  windowSize?: [number, number];
}) =>
  invoke<null>("layout_screenshot_preview_surface", {
    ...layout,
    annotationCounterAngle: annotationCounterAngle ?? null,
    annotationDefaults: annotationDefaults ?? null,
    annotationImage: annotationImage ?? null,
    annotationTextLook: annotationTextLook ?? null,
    annotationTool: annotationTool ?? null,
    annotationZoom: annotationZoom ?? null,
    interactionOutput: normalizedScreenshotWorkspaceOutput(interactionOutput),
    output: normalizedScreenshotWorkspaceOutput(output),
    selectedAnnotationIds,
    selection: selection ?? null,
    selectionTargets: selectionTargets ?? null,
  });

export const refreshScreenshotPreviewSources = (
  artifactId: number,
  sessionId: number,
) =>
  invoke<null>("refresh_screenshot_preview_sources", {
    artifactId,
    sessionId,
  });

export const stopScreenshotPreview = (sessionId: number) =>
  invoke<null>("stop_screenshot_preview", { sessionId });

export const setScreenshotPreviewEditorSuspended = (
  sessionId: number,
  suspended: boolean,
) =>
  invoke<null>("set_screenshot_preview_editor_suspended", {
    sessionId,
    suspended,
  });
