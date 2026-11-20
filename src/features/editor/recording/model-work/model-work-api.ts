// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { ModelWork } from "../../../../bindings/ModelWork";
import type { ModelWorkList } from "../../../../bindings/ModelWorkList";

export const getModelWork = (artifactId: number) =>
  invoke<ModelWorkList>("get_model_work", { artifactId });

/** Stops the work and takes it off the list; turning the tool off is the
 * caller's. */
export const cancelModelWork = (artifactId: number, work: ModelWork) =>
  invoke<null>("cancel_model_work", { artifactId, work });

export const listenToModelWork = (onList: (list: ModelWorkList) => void) =>
  listen<ModelWorkList>("editor://model-work", (event) => {
    onList(event.payload);
  });
