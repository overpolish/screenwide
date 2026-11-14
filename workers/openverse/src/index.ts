// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Openverse for Screenwide's Library island: the app asks here, and this
// asks Openverse with the app's registration, which never ships in the app,
// so no one can take it and use up the searches everyone shares. It answers
// searches (`/search`) and looks a work up again as it is downloaded
// (`/work`), so a work whose licence changed is not added. An answer already
// given is kept in the cache: a day for a search, an hour for a work, whose
// licence is what is being checked. Only a question sent on to Openverse
// counts against the asker's allowance (`allowance.ts`).

import { endOfDay } from "./allowance";
import { mayAsk } from "./allowance-object";
import { parseSearch, search, type Tokens, UpstreamError } from "./search";
import { keptToken } from "./token";
import { lookUp, parseWork } from "./work";

export { Allowance } from "./allowance-object";

/** How long a search's answer is kept, in s. */
const SEARCH_KEEP_S = 86_400;

/** How long a work's answer is kept, in s. */
const WORK_KEEP_S = 3_600;

/** Where an answer is kept in the cache: one address for each question,
 * however it was written. */
const keptAt = (path: string, asked: Record<string, string>) =>
  `https://openverse.screenwide.internal${path}?${new URLSearchParams(asked).toString()}`;

const answer = (body: unknown, status = 200, keepS = 0) =>
  Response.json(body, {
    headers: {
      "Cache-Control": keepS > 0 ? `max-age=${String(keepS)}` : "no-store",
    },
    status,
  });

/** The question `url` asks, in the words the cache keeps it under, and how
 * it is answered; or why it is not one. */
function questionOf(url: URL) {
  if (url.pathname === "/search") {
    const asked = parseSearch(url);
    if (typeof asked === "string") return asked;
    return {
      ask: (tokens: Tokens) =>
        search(asked, tokens, (input, init) => fetch(input, init)),
      keepS: SEARCH_KEEP_S,
      key: keptAt(url.pathname, {
        kind: asked.kind,
        page: String(asked.page),
        q: asked.query,
        ...asked.filters,
      }),
    };
  }
  if (url.pathname === "/work") {
    const asked = parseWork(url);
    if (typeof asked === "string") return asked;
    return {
      ask: async (tokens: Tokens) => ({
        found: await lookUp(asked, tokens, (input, init) => fetch(input, init)),
      }),
      keepS: WORK_KEEP_S,
      key: keptAt(url.pathname, asked),
    };
  }
  return null;
}

export default {
  async fetch(request, env, ctx): Promise<Response> {
    const url = new URL(request.url);
    const question = request.method === "GET" ? questionOf(url) : null;
    if (question === null) return answer({ error: "not_found" }, 404);
    if (typeof question === "string") return answer({ error: question }, 400);
    if (!env.OPENVERSE_CLIENT_ID || !env.OPENVERSE_CLIENT_SECRET)
      return answer({ error: "not_configured" }, 500);
    const cache = caches.default;
    const kept = await cache.match(question.key);
    if (kept) return kept;
    const verdict = await mayAsk(request, env);
    // A day's allowance used up says when it comes back, so the app can.
    if (verdict === "usedUp")
      return answer({ error: "used_up", until: endOfDay(Date.now()) }, 429);
    if (verdict === "busy") return answer({ error: "busy" }, 429);
    try {
      const body = await question.ask((fresh) =>
        keptToken(
          cache,
          {
            clientId: env.OPENVERSE_CLIENT_ID,
            clientSecret: env.OPENVERSE_CLIENT_SECRET,
          },
          fresh,
          (work) => {
            ctx.waitUntil(work);
          },
        ),
      );
      const answered = answer(body, 200, question.keepS);
      ctx.waitUntil(cache.put(question.key, answered.clone()));
      return answered;
    } catch (cause) {
      console.error(
        JSON.stringify({
          error: cause instanceof Error ? cause.message : String(cause),
          status: cause instanceof UpstreamError ? cause.status : null,
        }),
      );
      const isBusy = cause instanceof UpstreamError && cause.status === 429;
      return answer(
        { error: isBusy ? "busy" : "upstream" },
        isBusy ? 429 : 502,
      );
    }
  },
} satisfies ExportedHandler<Env>;
