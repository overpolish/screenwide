// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use, useEffect } from "react";

/**
 * Fitting the picture, from a toolbar button that is nowhere near the preview.
 *
 * The preview owns the transform and hands its own fit in; the tools that
 * reset the view call it back. A context rather than a module global, so each
 * editor window and each story fits its own preview and nothing else.
 */
export type PreviewFit = {
  /** Zoom to fit and pan reset. Omit width to use the current viewport. */
  fitPreview: (width?: number) => void;
  /** The preview's own fit, held until it unmounts. */
  registerFitPreview: (fit: (width?: number) => void) => () => void;
};

export const PreviewFitContext = createContext<PreviewFit>({
  fitPreview: () => undefined,
  registerFitPreview: () => () => undefined,
});

export const usePreviewFit = () => use(PreviewFitContext);

/** The preview's side of the arrangement: hands its fit to whoever asks. */
export function useRegisterPreviewFit(fit: (width?: number) => void) {
  const { registerFitPreview } = use(PreviewFitContext);
  useEffect(() => registerFitPreview(fit), [fit, registerFitPreview]);
}
