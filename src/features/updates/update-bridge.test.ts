// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { expect, it } from "vitest";

import { shouldRequestUpdateCheck } from "./update-bridge";

it("reuses recent successful checks", () => {
  expect(
    shouldRequestUpdateCheck({
      checkedAt: 100_000,
      now: 130_000,
      pending: false,
      status: "up-to-date",
    }),
  ).toBe(false);
  expect(
    shouldRequestUpdateCheck({
      checkedAt: 100_000,
      now: 160_000,
      pending: false,
      status: "available",
    }),
  ).toBe(true);
});
it("never duplicates an in-flight check or interrupts installation", () => {
  expect(
    shouldRequestUpdateCheck({
      checkedAt: 0,
      force: true,
      pending: true,
      status: "idle",
    }),
  ).toBe(false);
  expect(
    shouldRequestUpdateCheck({
      checkedAt: 0,
      force: true,
      pending: false,
      status: "checking",
    }),
  ).toBe(false);
  expect(
    shouldRequestUpdateCheck({
      checkedAt: 0,
      force: true,
      pending: false,
      status: "downloading",
    }),
  ).toBe(false);
});
it("allows explicit retries without waiting for the cache", () => {
  expect(
    shouldRequestUpdateCheck({
      checkedAt: 100_000,
      force: true,
      now: 100_001,
      pending: false,
      status: "error",
    }),
  ).toBe(true);
});
