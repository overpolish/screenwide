// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// One search of Openverse, as the app asks for it and as it is answered:
// only what may be used in a recording or screenshot with no more than a
// credit (public domain, CC0 and CC BY; never ShareAlike, NonCommercial or
// NoDerivatives, since music under a video adapts it and ShareAlike would
// bind the whole video), in the formats the app reads, cut down to what the
// Library island shows and the credit needs.

const API = "https://api.openverse.org/v1";

/** What may be used with no more than a credit. */
const LICENSES = "cc0,pdm,by";

/** Results a page: Openverse's most for one request. */
const PAGE_SIZE = 20;

/** The last page asked for: past it, Openverse answers nothing. */
const LAST_PAGE = 12;

/** The longest query Openverse takes. */
const LONGEST_QUERY = 200;

/** What the library shows of songs, of sound effects and of pictures, as
 * Openverse's own filters: a sound effect is under 30 seconds, a song from
 * 30 seconds to 10 minutes. */
const KINDS = {
  effect: {
    endpoint: "audio",
    filters: {
      extension: "mp3,ogg,wav,flac,m4a,aac,aiff,opus",
      length: "shortest",
    },
  },
  image: {
    endpoint: "images",
    filters: { extension: "jpg,jpeg,png,gif,webp" },
  },
  music: {
    endpoint: "audio",
    filters: {
      extension: "mp3,ogg,wav,flac,m4a,aac,aiff,opus",
      length: "short,medium",
    },
  },
} as const;

export type Kind = keyof typeof KINDS;

/** Openverse's endpoint for `kind`'s works. */
export const endpointOf = (kind: Kind) => `${API}/${KINDS[kind].endpoint}/`;

/** Whether `kind` is a kind of work the app asks for. */
export const isKind = (kind: string | null): kind is Kind =>
  kind === "music" || kind === "effect" || kind === "image";

export type Search = { kind: Kind; page: number; query: string };

/** One result, as the app takes it: enough to show it, fetch it and credit
 * it. */
export type Found = {
  /** The credit Openverse writes for it. */
  attribution: string;
  creator: string | null;
  creatorUrl: string | null;
  /** How long it plays, in ms; none for a picture. */
  durationMs: number | null;
  filetype: string | null;
  height: number | null;
  id: string;
  /** Its page where it is published. */
  landingUrl: string | null;
  license: string;
  licenseUrl: string | null;
  licenseVersion: string | null;
  /** A small picture of it, for a picture. */
  thumbnail: string | null;
  title: string;
  /** The file itself. */
  url: string;
  width: number | null;
};

export type Page = { found: Found[]; pageCount: number };

/** The search `url` asks for, or why it is not one. */
export function parseSearch(url: URL): Search | string {
  const kind = url.searchParams.get("kind");
  const query = (url.searchParams.get("q") ?? "").trim().replace(/\s+/g, " ");
  const page = Number(url.searchParams.get("page") ?? "1");
  if (!isKind(kind)) return "unknown_kind";
  if (query.length === 0 || query.length > LONGEST_QUERY) return "bad_query";
  if (!Number.isInteger(page) || page < 1 || page > LAST_PAGE)
    return "bad_page";
  return { kind, page, query: query.toLowerCase() };
}

/** The Openverse request for `search`. */
export function upstreamUrl({ kind, page, query }: Search) {
  const { filters } = KINDS[kind];
  const url = new URL(endpointOf(kind));
  url.search = new URLSearchParams({
    ...filters,
    license: LICENSES,
    mature: "false",
    page: String(page),
    page_size: String(PAGE_SIZE),
    q: query,
  }).toString();
  return url;
}

const text = (value: unknown) =>
  typeof value === "string" && value.length > 0 ? value : null;
const number = (value: unknown) =>
  typeof value === "number" && Number.isFinite(value) ? value : null;

/** One of Openverse's results as the app takes it, or none where it lacks a
 * file or a licence the app can use. */
export function foundOf(result: unknown): Found | null {
  if (!result || typeof result !== "object") return null;
  const field = (name: string): unknown => Reflect.get(result, name);
  const url = text(field("url"));
  const license = text(field("license"));
  const id = text(field("id"));
  if (!url || !id || !license || !LICENSES.split(",").includes(license))
    return null;
  return {
    attribution: text(field("attribution")) ?? "",
    creator: text(field("creator")),
    creatorUrl: text(field("creator_url")),
    durationMs: number(field("duration")),
    filetype: text(field("filetype")),
    height: number(field("height")),
    id,
    landingUrl: text(field("foreign_landing_url")),
    license,
    licenseUrl: text(field("license_url")),
    licenseVersion: text(field("license_version")),
    thumbnail: text(field("thumbnail")),
    title: text(field("title")) ?? id,
    url,
    width: number(field("width")),
  };
}

/** Openverse's answer to a search, cut down to what the app takes. */
function pageOf(body: unknown): Page {
  if (!body || typeof body !== "object") return { found: [], pageCount: 0 };
  const results: unknown = Reflect.get(body, "results");
  const pageCount = number(Reflect.get(body, "page_count")) ?? 0;
  return {
    found: Array.isArray(results)
      ? results.flatMap((result) => foundOf(result) ?? [])
      : [],
    pageCount: Math.min(pageCount, LAST_PAGE),
  };
}

/** A token for Openverse: the one kept, or a new one when `fresh`. */
export type Tokens = (fresh: boolean) => Promise<string>;

/** Asks Openverse for `url` with the app's token, asking for a new token
 * once where the one kept has lapsed. */
export async function askOpenverse(
  url: URL,
  tokens: Tokens,
  fetcher: (input: URL, init: RequestInit) => Promise<Response>,
) {
  const ask = async (fresh: boolean) =>
    fetcher(url, {
      headers: {
        Accept: "application/json",
        Authorization: `Bearer ${await tokens(fresh)}`,
      },
    });
  const answer = await ask(false);
  return answer.status === 401 ? ask(true) : answer;
}

/** Searches Openverse for `search` with the app's token. */
export async function search(
  request: Search,
  tokens: Tokens,
  fetcher: (input: URL, init: RequestInit) => Promise<Response>,
): Promise<Page> {
  const answer = await askOpenverse(upstreamUrl(request), tokens, fetcher);
  if (!answer.ok) throw new UpstreamError(answer.status);
  return pageOf(await answer.json());
}

/** Openverse answered with something other than results. */
export class UpstreamError extends Error {
  constructor(readonly status: number) {
    super(`Openverse answered ${String(status)}`);
  }
}
