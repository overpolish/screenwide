// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { SourceRect } from "../screenshot-geometry";
import { ScreenshotOutputSettings } from "../screenshot-output";
import {
  getScreenshotRecenterAnalysis,
  recenterScreenshotContent,
  resetScreenshotRecenter,
  ScreenshotRecenterAnalysis,
} from "../screenshot-recenter";

export function useScreenshotRecenter({
  artifactId,
  onOutputChange,
  selectedItem,
  selectedOutput,
}: {
  artifactId: number;
  selectedOutput: ScreenshotOutputSettings | null;
  onOutputChange?: (
    settings: ScreenshotOutputSettings,
    itemId?: number,
  ) => void;
  selectedItem?: { height: number; id: number; width: number };
}) {
  const analysisKey = (itemId: number, crop: SourceRect) =>
    [itemId, crop.x, crop.y, crop.width, crop.height].join(":");
  const analysesRef = useRef(new Map<number, symbol>());
  const resultsRef = useRef(
    new Map<string, Promise<ScreenshotRecenterAnalysis | null>>(),
  );
  const selectedRef = useRef({ id: selectedItem?.id, output: selectedOutput });
  selectedRef.current = { id: selectedItem?.id, output: selectedOutput };
  const analyse = (itemId: number, crop: SourceRect, refresh = false) => {
    const key = analysisKey(itemId, crop);
    const cached = !refresh && resultsRef.current.get(key);
    if (cached) return cached;
    const result = getScreenshotRecenterAnalysis(artifactId, itemId, crop);
    resultsRef.current.set(key, result);
    return result;
  };
  const begin = () => {
    if (!selectedItem || !selectedOutput) return;
    const analysisToken = Symbol();
    const key = analysisKey(selectedItem.id, selectedOutput.sourceCrop);
    analysesRef.current.set(selectedItem.id, analysisToken);
    analyse(selectedItem.id, selectedOutput.sourceCrop, true)
      .then((analysis) => {
        if (
          !analysis ||
          analysesRef.current.get(selectedItem.id) !== analysisToken
        )
          return;
        const currentOutput =
          selectedRef.current.id === selectedItem.id
            ? selectedRef.current.output
            : selectedOutput;
        if (
          !currentOutput ||
          analysisKey(selectedItem.id, currentOutput.sourceCrop) !== key
        )
          return;
        const colored = {
          ...currentOutput,
          recenterInsetColor: analysis.backgroundColor,
        };
        onOutputChange?.(
          analysis.bounds
            ? recenterScreenshotContent(colored, selectedItem, analysis.bounds)
            : colored,
          selectedItem.id,
        );
      })
      .catch((error: unknown) => {
        if (analysesRef.current.get(selectedItem.id) === analysisToken)
          analysesRef.current.delete(selectedItem.id);
        console.error("Could not detect screenshot content bounds", error);
      });
  };
  const prepare = () => {
    if (!selectedItem || !selectedOutput || selectedOutput.recenterInsetColor)
      return;
    const itemId = selectedItem.id;
    const key = analysisKey(itemId, selectedOutput.sourceCrop);
    analyse(itemId, selectedOutput.sourceCrop)
      .then((analysis) => {
        if (!analysis || selectedRef.current.id !== itemId) return;
        const current = selectedRef.current.output;
        if (
          !current ||
          current.recenterInsetColor ||
          analysisKey(itemId, current.sourceCrop) !== key
        )
          return;
        onOutputChange?.(
          { ...current, recenterInsetColor: analysis.backgroundColor },
          itemId,
        );
      })
      .catch((error: unknown) => {
        resultsRef.current.delete(key);
        console.error("Could not detect screenshot inset colour", error);
      });
  };
  const refresh = (crop: SourceRect) => {
    const itemId = selectedRef.current.id;
    if (itemId === undefined) return;
    const analysisToken = Symbol();
    const key = analysisKey(itemId, crop);
    analysesRef.current.set(itemId, analysisToken);
    analyse(itemId, crop, true)
      .then((analysis) => {
        if (
          !analysis ||
          analysesRef.current.get(itemId) !== analysisToken ||
          selectedRef.current.id !== itemId
        )
          return;
        const current = selectedRef.current.output;
        if (!current || analysisKey(itemId, current.sourceCrop) !== key) return;
        onOutputChange?.(
          { ...current, recenterInsetColor: analysis.backgroundColor },
          itemId,
        );
      })
      .catch((error: unknown) => {
        if (analysesRef.current.get(itemId) === analysisToken)
          analysesRef.current.delete(itemId);
        resultsRef.current.delete(key);
        console.error("Could not refresh screenshot inset colour", error);
      });
  };
  const reset = () => {
    if (!selectedItem || !selectedOutput) return;
    analysesRef.current.delete(selectedItem.id);
    onOutputChange?.(
      resetScreenshotRecenter(selectedOutput, selectedItem),
      selectedItem.id,
    );
  };
  return { begin, prepare, refresh, reset };
}
