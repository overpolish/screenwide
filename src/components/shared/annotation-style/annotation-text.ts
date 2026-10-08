// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { AnnotationKind } from "./types";

/** What an annotation's size control measures. */
export type AnnotationSizeMeasure = "blockSize" | "height" | "size" | "width";

/** The name a drawing tool goes by, the same in the editor and the live
 * overlay so an annotation reads the same wherever it is drawn. */
export const annotationToolLabel = (kind: AnnotationKind) => {
  switch (kind) {
    case "arrow":
      return t("annotation-tool-arrow");
    case "counter":
      return t("annotation-tool-counter");
    case "draw":
      return t("annotation-tool-draw");
    case "highlight":
      return t("annotation-tool-highlight");
    case "image":
      return t("annotation-tool-image");
    case "magnify":
      return t("annotation-tool-magnify");
    case "redact":
      return t("annotation-tool-redact");
    case "shape":
      return t("annotation-tool-shape");
    case "spotlight":
      return t("annotation-tool-spotlight");
    case "text":
      return t("annotation-tool-text");
  }
};

/** A drawing tool's accessible name, which says what it acts on. */
export const annotationToolName = (kind: AnnotationKind) => {
  switch (kind) {
    case "arrow":
      return t("annotation-tool-arrow-name");
    case "counter":
      return t("annotation-tool-counter-name");
    case "draw":
      return t("annotation-tool-draw-name");
    case "highlight":
      return t("annotation-tool-highlight-name");
    case "image":
      return t("annotation-tool-image-name");
    case "magnify":
      return t("annotation-tool-magnify-name");
    case "redact":
      return t("annotation-tool-redact-name");
    case "shape":
      return t("annotation-tool-shape-name");
    case "spotlight":
      return t("annotation-tool-spotlight-name");
    case "text":
      return t("annotation-tool-text-name");
  }
};

/** What an annotation's size control is called. */
export const annotationSizeLabel = (measure: AnnotationSizeMeasure) => {
  switch (measure) {
    case "blockSize":
      return t("annotation-size-block");
    case "height":
      return t("annotation-size-height");
    case "size":
      return t("annotation-size-size");
    case "width":
      return t("annotation-size-width");
  }
};
