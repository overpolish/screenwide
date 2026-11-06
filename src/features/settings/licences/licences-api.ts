// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { AddKeyOutcome } from "../../../bindings/AddKeyOutcome";
import type { Checkout } from "../../../bindings/Checkout";
import type { Licences } from "../../../bindings/Licences";
import type { PaidTool } from "../../../bindings/PaidTool";

export const getLicences = () => invoke<Licences>("get_licences");

export const addLicenceKey = (key: string) =>
  invoke<AddKeyOutcome>("add_licence_key", { key });

export const removeLicenceKey = (id: string) =>
  invoke<Licences>("remove_licence_key", { id });

export const openLicenceCheckout = (checkout: Checkout) =>
  invoke<null>("open_licence_checkout", { checkout });

/** Fired whenever the licences change: a key added, removed or refused at
 * its check, or the trial started. */
export const listenToLicences = (onChange: (licences: Licences) => void) =>
  listen<Licences>("licences://changed", (event) => {
    onChange(event.payload);
  });

/** Fired when a paid tool was turned away and Settings should show the
 * Licences pane, pointing at that tool. */
export const listenToLicencesOpen = (onOpen: (tool: PaidTool | null) => void) =>
  listen<{ tool: PaidTool | null }>("licences://open", (event) => {
    onOpen(event.payload.tool);
  });
