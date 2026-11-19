// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationTool } from "../annotations/annotation-defaults";
import type { CaptureSource } from "../layer-measure";
import type { PreviewPaneFit } from "./preview-transform";
import type { SourceRect } from "../screenshot/screenshot-geometry";
import type {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot/screenshot-output";

export type PreviewViewportProps = {
  alt: string;
  artifactId: number;
  /** The screenshot's layers as captured. */
  items: (CaptureSource & { id: number })[];
  naturalHeight: number;
  naturalWidth: number;
  /** The annotation tool in hand. "select" hit-tests the annotations already on
   * the layer; "arrow" and "counter" also make a new one on empty picture,
   * and the layer's own chrome stands down for as long as one is held. */
  annotationTool?: AnnotationTool;
  /** The crop in hand keeps its shape as its handles are dragged. */
  cropKeepsShape?: boolean;
  isEditing?: boolean;
  /** The Export island stands over the editor: native input stands down
   * under it, open, exporting or done. */
  isExportOpen?: boolean;
  isResizingCanvas?: boolean;
  /** ⌘ (Ctrl) holds Select over the tool: the chosen layer's chrome stands
   * down and a press on it draws a band. */
  isSelectHeld?: boolean;
  isSelecting?: boolean;
  onAnnotationHover?: (annotationId: string | null) => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onCropChangeEnd?: (crop: SourceRect) => Promise<void>;
  onItemSelect?: (itemId: number | null) => void;
  onOutputChange?: (
    settings: ScreenshotOutputSettings,
    itemId?: number,
  ) => void;
  onPaneFitChange?: (fit: PreviewPaneFit) => void;
  onSelectedAnnotationsChange?: (annotationIds: readonly string[]) => void;
  screenshotOutput?: ScreenshotWorkspaceOutputSettings;
  selectedAnnotationIds?: readonly string[];
  selectedItemId?: number | null;
};
