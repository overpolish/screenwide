// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use, useEffect, useSyncExternalStore } from "react";

type Dimensions = { height: number; width: number };
type Listener = () => void;

export const createRecordingOutputDimensionsChannel = () => {
  const listeners = new Set<Listener>();
  let latest: Dimensions | null = null;
  return {
    getSnapshot: () => latest,
    publish: (dimensions: Dimensions) => {
      if (
        latest?.height === dimensions.height &&
        latest.width === dimensions.width
      )
        return;
      latest = dimensions;
      for (const listener of listeners) listener();
    },
    subscribe: (listener: Listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
};

export type RecordingOutputDimensionsChannel = ReturnType<
  typeof createRecordingOutputDimensionsChannel
>;
export const RecordingOutputDimensionsContext =
  createContext<RecordingOutputDimensionsChannel | null>(null);
const emptySubscribe = () => () => undefined;
const emptySnapshot = (): Dimensions | null => null;

export const useRecordingOutputDimensions = () => {
  const channel = use(RecordingOutputDimensionsContext);
  return useSyncExternalStore<Dimensions | null>(
    channel?.subscribe ?? emptySubscribe,
    channel?.getSnapshot ?? emptySnapshot,
  );
};

export const usePublishRecordingOutputDimensions = ({
  height,
  width,
}: Dimensions) => {
  const channel = use(RecordingOutputDimensionsContext);
  useEffect(() => {
    channel?.publish({ height, width });
  }, [channel, height, width]);
};
