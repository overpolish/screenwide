// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { beforeEach, expect, it, vi } from "vitest";

// Refs persist across renders by call order, as React keeps them.
const refs = vi.hoisted(() => ({
  index: 0,
  store: [] as { current: unknown }[],
}));
vi.mock("react", () => ({
  useRef: (initial: unknown) => {
    refs.store[refs.index] ??= { current: initial };
    return refs.store[refs.index++];
  },
}));

import { useSelectToolMemory } from "./use-select-tool-memory";

const choose = vi.fn();
const Rendered = (...ids: string[]) => {
  refs.index = 0;
  return useSelectToolMemory(new Set(ids), choose);
};

beforeEach(() => {
  refs.store = [];
  choose.mockReset();
});

it("chooses the annotations again when select is put down and taken back up", () => {
  Rendered("a", "b")("select", null);
  Rendered()(null, "select");
  expect(choose).toHaveBeenCalledExactlyOnceWith(["a", "b"]);
});

it("keeps them through other tools taken up in between", () => {
  Rendered("a")("select", "pen");
  Rendered()("pen", "crop");
  Rendered()("crop", "select");
  expect(choose).toHaveBeenCalledExactlyOnceWith(["a"]);
});

it("never puts them back over a choice made in the meantime", () => {
  Rendered("a")("select", null);
  Rendered("b")(null, "select");
  expect(choose).not.toHaveBeenCalled();
});

it("puts them back once only", () => {
  Rendered("a")("select", null);
  Rendered()(null, "select");
  Rendered()("select", null);
  Rendered()(null, "select");
  expect(choose).toHaveBeenCalledOnce();
});
