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
  /** The preview's own fit and basis, held until it unmounts. */
  registerFitPreview: (
    fit: (width?: number) => void,
    setBasis: (width?: number) => void,
  ) => () => void;
  /**
   * The width a double-click reset fits into from now on, without moving the
   * view. A fit already leaves its own width behind as the basis; this is for
   * handing it back when a tool panel closes. Omit the width for the full
   * viewport.
   */
  setFitBasis: (width?: number) => void;
};

export const PreviewFitContext = createContext<PreviewFit>({
  fitPreview: () => undefined,
  registerFitPreview: () => () => undefined,
  setFitBasis: () => undefined,
});

export const usePreviewFit = () => use(PreviewFitContext);

/** The preview's side of the arrangement: hands its fit to whoever asks. */
export function useRegisterPreviewFit(
  fit: (width?: number) => void,
  setBasis: (width?: number) => void,
) {
  const { registerFitPreview } = use(PreviewFitContext);
  useEffect(
    () => registerFitPreview(fit, setBasis),
    [fit, registerFitPreview, setBasis],
  );
}
