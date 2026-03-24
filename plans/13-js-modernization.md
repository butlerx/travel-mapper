# 13 — JavaScript Modernization

## What

Audit and modernize the frontend JavaScript to improve maintainability, type safety, and security — without adding operational complexity (no npm, no node_modules, no bundler).

## Current State

**4 static JS files** (~34KB, 1,309 LOC total):

| File | LOC | Size | Role | ES Level |
|------|-----|------|------|----------|
| `static/nav.js` | 31 | 935B | Mobile nav toggle | Modern-ish (const, arrow fns) |
| `static/sw.js` | 73 | 1.8KB | Service worker (PWA cache) | Modern (const, arrow fns, Promises) |
| `static/map.js` | 768 | 21.6KB | Dashboard Leaflet map | **Legacy** (var, no modules, string concat) |
| `static/stats-map.js` | 437 | 9.9KB | Stats choropleth map | **Legacy** (var, no modules) |

**Inline JS in Leptos templates** (~65 LOC across 4 files):

| Source file | What | LOC |
|-------------|------|-----|
| `src/server/components/shell.rs` | SW registration | 1 |
| `src/server/pages/dashboard.rs` | `window.allHops = <json>` injection | 1 + payload |
| `src/server/pages/stats.rs` | `window.countryCounts = <json>` injection | 1 + payload |
| `src/server/pages/hop_detail.rs` | Single-hop Leaflet map init | ~22 |
| `src/server/pages/add_hop.rs` | Form section toggling by travel type | ~40 |

**External CDN dependencies** (loaded via `<script>` tags in page templates):

- Leaflet JS + CSS (unpkg CDN)
- topojson-client (jsdelivr CDN)
- world-atlas topojson data (jsdelivr CDN)

## Chosen Approach: ES Modules + JSDoc Types

**Why this over alternatives:**

| Option | Ops Complexity | Type Safety | Build Step | Verdict |
|--------|---------------|-------------|------------|---------|
| ES Modules + JSDoc | Zero | Very Good (90% of TS) | None | **Selected** |
| esbuild + TypeScript | Low-Med (8MB binary) | Good (no type checking without tsc) | mise task | Maybe later |
| SWC | Medium (37MB, no bundling) | Good | mise task | No |
| Leptos WASM | Med-High (trunk/cargo-leptos) | Excellent | cargo-leptos | No (too complex) |
| Deno TS | Medium (80MB, browsers can't run TS) | Excellent | N/A | No |

ES Modules + JSDoc preserves zero-JS-tooling, works with `include_str!()`, and adds real type safety through VS Code IntelliSense and optional `tsc --checkJs --noEmit` in CI.

## Scope

### Phase 1 — Security & Extraction (do first)

1. **Extract inline scripts to external files**
   - `src/server/pages/add_hop.rs` inline script -> `static/add-hop.js`
   - `src/server/pages/hop_detail.rs` map_script -> `static/hop-map.js`
   - Update Leptos templates to reference new external files
   - Keeps `include_str!()` pattern intact

2. **Add HTML escaping for XSS prevention**
   - `map.js` builds popup HTML via string concatenation with unescaped user data
   - `stats-map.js` does the same for tooltips
   - Create a shared `escapeHtml()` utility and apply to all innerHTML/popup content

3. **Replace global `window.*` data injection**
   - Change `window.allHops = <json>` to `<script type="application/json" id="initial-hops">...</script>`
   - Change `window.countryCounts = <json>` similarly
   - Update map.js / stats-map.js to read from `JSON.parse(document.getElementById(...).textContent)`
   - Removes need for `unsafe-inline` in future CSP

### Phase 2 — Modernize to ES Modules

4. **Convert `map.js` to ES module(s)**
   - Split 768-line IIFE into logical modules: `map-utils.js`, `map-render.js`, `map-filters.js`, `map-popups.js`
   - Replace `var` with `let`/`const` throughout
   - Replace string concatenation with template literals
   - Replace `.then()` chains with `async`/`await`
   - Load via `<script type="module" src="/static/map.js"></script>`

5. **Convert `stats-map.js` to ES module(s)**
   - Same modernization: `let`/`const`, template literals, async/await
   - Consider splitting fetch/render logic

6. **Update `nav.js` and `sw.js`**
   - Already mostly modern; minor cleanup (add `// @ts-check`, JSDoc types)
   - `sw.js` cannot be a module (service workers have different loading), keep as classic script

### Phase 3 — Add JSDoc Type Safety

7. **Add JSDoc type definitions**
   - Create `static/types.js` with shared `@typedef` definitions (`Hop`, `Route`, `Airport`, etc.)
   - Add `// @ts-check` directive to all JS files
   - Annotate all function signatures with `@param` / `@returns`
   - Use `@import` (TS 5.5+) to share type definitions across modules

8. **Optional CI type checking**
   - Add `mise run typecheck` task: `tsc --checkJs --noEmit --target ES2022 --module ESNext static/*.js`
   - Requires `typescript` as a dev dependency (npm install) — defer until rest is done
   - Could alternatively use Deno for type checking without npm

## Files to Change

### Phase 1
- `static/add-hop.js` — **new** (extracted from add_hop.rs)
- `static/hop-map.js` — **new** (extracted from hop_detail.rs)
- `static/map.js` — add `escapeHtml()`, read data from `<script type="application/json">`
- `static/stats-map.js` — add `escapeHtml()`, read data from `<script type="application/json">`
- `src/server/pages/add_hop.rs` — replace inline script with `<script src="/static/add-hop.js">`
- `src/server/pages/hop_detail.rs` — replace inline script with `<script src="/static/hop-map.js">`
- `src/server/pages/dashboard.rs` — change `window.allHops` to `<script type="application/json">`
- `src/server/pages/stats.rs` — change `window.countryCounts` to `<script type="application/json">`
- `src/server/routes/static_assets.rs` — add `include_str!()` entries for new files

### Phase 2
- `static/map.js` — rewrite as ES module, split into sub-modules
- `static/map-utils.js` — **new** (haversine, arc, distance helpers)
- `static/map-render.js` — **new** (marker/line rendering)
- `static/map-filters.js` — **new** (URL filter sync, UI controls)
- `static/map-popups.js` — **new** (popup HTML generation)
- `static/stats-map.js` — modernize syntax
- `static/nav.js` — minor cleanup
- `src/server/pages/dashboard.rs` — change `<script>` to `<script type="module">`
- `src/server/pages/stats.rs` — change `<script>` to `<script type="module">`
- `src/server/routes/static_assets.rs` — add new module files

### Phase 3
- `static/types.js` — **new** (shared JSDoc type definitions)
- All `static/*.js` files — add `// @ts-check` and JSDoc annotations
- `.mise.toml` — optional `typecheck` task

## Dependencies

- None for Phases 1-3 (zero new runtime/build dependencies)
- Optional: `typescript` npm package for CI type checking only
- No blocking dependencies on other plan items

## Acceptance Criteria

### Phase 1
- [ ] No inline `<script>` blocks remain in Leptos templates (except SW registration one-liner in shell.rs and CDN includes)
- [ ] All `innerHTML` / popup content uses `escapeHtml()` for user-provided strings
- [ ] No `window.*` global variable injection — data passed via `<script type="application/json">`
- [ ] All existing functionality works identically (maps, filters, nav, forms)

### Phase 2
- [ ] `map.js` split into 4+ focused modules, each under 200 LOC
- [ ] Zero `var` declarations across all static JS
- [ ] All files use `const`/`let`, template literals, `async`/`await` where applicable
- [ ] Scripts load as `type="module"` (except `sw.js` and `nav.js`)
- [ ] Browser console shows no errors on dashboard, stats, hop detail, add hop pages

### Phase 3
- [ ] All JS files have `// @ts-check` directive
- [ ] All exported functions have JSDoc `@param` and `@returns` annotations
- [ ] Shared types defined in `types.js` and imported where used
- [ ] VS Code shows no type errors with TypeScript language server
- [ ] Optional: `mise run typecheck` passes (if CI checking added)
