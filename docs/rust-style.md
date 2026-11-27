# Rust Style

Rust conventions for services, shared libraries and tools. A project's
`AGENTS.md` can point here; project files add only what is specific to them.
Examples use a made-up `orders` domain.

These services handle millions of requests. The default is code that does no
work it doesn't need to: no copy it can borrow, no allocation it can avoid, no
state it can express as a value.

## Basics

- Lints are configured in `Cargo.toml`, not with `#![warn(…)]` in crate
  roots: `[lints.clippy]` in a single crate, or `[workspace.lints.clippy]`
  with `lints.workspace = true` in each member. `pedantic` is on
  (`{ level = "warn", priority = -1 }`); CI runs
  `cargo clippy --all-targets --all-features -- -D warnings`.
- Modules use `module.rs` + `module/`, never `mod.rs`.
- Format with `cargo fmt`; test with `cargo nextest`.
- `.expect("why it can't fail")` documents an invariant; never a bare
  `.unwrap()` outside tests.

## Types

Make wrong states unrepresentable, and let the compiler carry the facts.

- **Enums over flags and strings.** A request is `Action::Cancel`, not
  `is_cancel: bool`; an outcome is `Outcome::Created`, not `"created"` passed
  around. Give the enum the one function that turns it into text
  (`label() -> &'static str`).
- **Newtypes for values that mean something.** `AccountId(String)` can't be
  mixed up with an `OrderId(String)`; a response extension keyed by type can't collide.
- **Typestate builders** when a value needs several required parts
  (`ClientBuilder<Url, Auth, Retry>`): `build()` exists only once
  everything is set, so a missing field is a compile error, not a runtime one.
- **Borrowing types for request data.** A value that only lives as long as a
  request borrows from it: `Target<'a> { account: &'a str, … }`,
  `Classified<'a>`. Add an `into_owned()` for the rare caller that must keep
  it.
- **`Cow<'a, str>`** where a value is usually borrowed but sometimes has to
  be rewritten (a percent-decoded query value, a key reduced from a URL).
- **Traits for what varies between services**, generic or `PhantomData`, not
  configuration flags: tower-telemetry's `SpanDecorator` and `RequestFilter`
  on its trace layer, a `Store` trait over the storage backends. Shared code stays free of service
  names.
- **Pure core, effects at the edge.** Rules that decide (classify a request,
  parse a location) are plain functions over borrowed input, unit-tested
  exhaustively; I/O wraps them.
- **Constructors are `new`, `parse` or `from_*`.** A function that builds
  `Self` from its inputs, fallible or not, is named for where it comes from:
  `String::from_utf8`, `Action::from_request`, `OrderId::parse`. Not `for_*`,
  which reads as a lookup on an existing value.
- **Parse into checked types at the boundary.** A value from a client becomes
  a type that can only be built through its check (`OrderId`, `Quantity`),
  keeping the client's text where it must be passed on as sent; code past the
  boundary takes the type, not a `&str`.
- **Public API is deliberate.** `pub(crate)` by default; make an item `pub`
  when another crate needs it, with docs and `#[must_use]` where it applies.

## Functional style

- **Iterator chains over loops with accumulators.** A `for` loop that pushes
  into a collection is a `.collect()`; `HeaderMap`, `Vec`, `String` and
  `Result<C, E>` all collect.

  ```rust
  // Not this
  let mut headers = HeaderMap::new();
  for name in FORWARDED {
      if let Some(value) = client.get(name) {
          headers.insert(name.clone(), value.clone());
      }
  }

  // This
  let headers: HeaderMap = FORWARDED
      .iter()
      .filter_map(|name| client.get(*name).map(|value| ((*name).clone(), value.clone())))
      .collect();
  ```

- **Combinators over rewrapping.** `.map()`, `.and_then()`, `.map_err()`,
  `.ok_or()`, `.transpose()`, `.unwrap_or_else()` instead of a `match` that
  only rewraps.
- **Expressions, not mutate-then-return.** Build a response as one value
  (`(status, AppendHeaders(header), Json(body)).into_response()`), not
  `let mut response = …; response.headers_mut().insert(…); response`.
- **`let mut` only where an API demands it**, scoped as tightly as possible
  (an iterator you call `next()` on, a URL builder, an SDK `set_*` method).
  Bounded retries are `for attempt in 1..=ATTEMPTS` with a labelled block that
  breaks with the value, not a hand-kept counter.
- **Struct update** for "the same but with one field": `Self { field, ..self }`.

## Error handling

- **`thiserror` everywhere; never `anyhow`.** Each crate has its own error
  types; a service's top-level error wraps them with `#[from]` instead of
  restating their variants.
- **An error type per failure surface**, sized to what the caller does with
  it: `BuildError` (startup), `ApiError` (a call), `StoreError` (a write),
  `Rejection` (refuse the request). Don't make callers match variants they
  can't act on.
- **Answers are values, not errors.** A `401` from an upstream is a
  `Reply::Rejected`, not an `Err`: errors are for failing to get an answer.
- **Errors derive `thiserror::Error`, with the message in `#[error(…)]`.**
  Its `Display` is the text a client or log gets; a fixed message costs no
  allocation, and fields fill in the rest
  (`#[error("{} must be a whole number", .0.name(…))]`). Give the error a
  stable `code() -> &'static str` when clients branch on it.
- **Classify for metrics in one place** (`ClientError::labels()`,
  `RequestOutcome::label()`), with low-cardinality `&'static str` labels.
- **Log with structured fields** (`tracing::warn!(error = %err, order_id, …)`),
  once, where the error is handled, not at every layer it passes through.
- **Secrets never reach `Debug`.** Config holding keys or tokens has a
  hand-written `Debug` that redacts them, and so does any type holding
  `Authorization` or cookie headers.

## Memory and allocation

Measure the hot path in allocations per request, and keep it small.

- **Borrow request data; don't copy it.** Header values are read as `&str`
  from the `HeaderMap`; path segments are slices of the path; query values are
  decoded lazily, only for the keys asked for, as `Cow`.
- **Move, don't clone.** Take a value out when you're done with it:
  `Option::take()`, `std::mem::take`, destructuring
  (`let Order { id, .. } = order`), passing ownership into
  the response. A `.clone()` on a hot path needs a reason.
- **Clone only what is cheap to clone, and know which those are.** `Bytes`,
  `HeaderValue`, `HeaderName`, `Arc<T>`, `tracing::Span` and `http::Uri` are
  reference-counted or static; `String`, `Vec`, `HeaderMap` and `Url` are not.
- **Build a collection once.** Chain the sources into one `collect()` rather
  than cloning one map and extending it with a clone of another.
- **Static names, not parsed ones.** Header names are
  `static HeaderName`s defined once (`headers.get(&X_ACCOUNT_ID)`), not
  strings parsed on every lookup. Span names and metric labels
  known at compile time are `&'static str`, not `format!`.
- **No `format!` or `to_string()` on the hot path** for values that are fixed
  (`"orders.create"`, `if retryable { "true" } else { "false" }`). Serialize
  typed structs (`#[derive(Serialize)] struct CreateOrder<'a>`), not
  `serde_json::json!` values.
- **Stream bodies; never buffer them whole.** A large body goes from the
  request to its destination frame by frame; reads that must be whole are bounded
  (`MAX_WHOLE_BODY`) and pre-sized from the declared length.
- **Parse once.** Turn a URL into a `Uri` before a retry loop, not on every
  attempt.
- **Process-wide state is a `static`** (`LazyLock`, `OnceLock`): metric
  handles, hooks, exporters. Cache hot metric handles rather than rebuilding
  label sets per call when labels are bounded.
- **Bounded everything.** Buffers, queues, pools, retries and timeouts all
  have limits; an unbounded one is a memory leak waiting for load.
- **`#[inline]` small public functions other crates call.** Without LTO a
  non-generic function only inlines inside its own crate, so a library's
  accessors and lookups (`OrderId::as_str`, `Rejection::code`,
  `Caller::account`) get `#[inline]`. Leave it off private functions, generic
  ones (already instantiated in the caller), anything more than a few lines,
  and binary crates; the compiler decides better there.

## Async

- **Don't hold borrowed iterators or closures across `.await`.** An
  iterator of borrowed tuples living in an `async fn` can make the future fail
  to be `Send` for every lifetime ("one type is more general than the other").
  Do the borrowing work in a synchronous helper and pass slices.
- **Never block the runtime.** No synchronous I/O, `std::thread::sleep` or
  long CPU work in async code; the log writer runs on its own thread for this
  reason.
- **Cancellation is normal.** A dropped future (a client that went away) must
  leave counters and gauges right: track in-flight work with guards that
  decrement in `Drop`.
- **Spans follow futures** with `.instrument(span)`; never hold
  `span.enter()` across an `.await`.

## Database queries

Code that talks to a database through `sqlx` follows
[rust-sqlx.md](rust-sqlx.md). In short:

- One query, one struct: inputs are fields, `execute(&self, pool)` runs it,
  named verb-noun under its table's module (`db::projects::GetAll`).
- Compile-time checked SQL only (`query!`, `query_as!`, `query_scalar!`);
  runtime `sqlx::query()` only for PRAGMAs. Commit `.sqlx/` after
  `cargo sqlx prepare`.
- Each table's ID is a newtype (`ProjectId(i64)`, `#[sqlx(transparent)]`),
  never a bare `i64` or a type alias, so IDs can't be swapped.
- Scope every query by its owner, and give list queries a deterministic
  `ORDER BY`.
- Return `Result<T, sqlx::Error>`; updates and deletes return `bool` so a
  handler maps `false` to 404 without a second query.
- Several statements that must succeed together run in one transaction.

## Axum servers

An HTTP service on axum follows [rust-axum.md](rust-axum.md). In short:

- Split by layer, then by resource: `routes/<group>.rs` (handlers, docs,
  bodies), `services/<domain>.rs` (what the service does, no axum),
  `middleware/`, `extractors/`, `models/` and `clients/` when they have
  something in them, `state.rs`. Routes call services; services never import
  routes or axum.
- `main.rs` only wires; `router(state, &server)` merges each group's `routes()` and is
  separate from `serve` so tests drive the real router.
- One `Clone` state of services and policies, wrapped in `Arc` once, with
  private fields and `&` accessors; test with real clients against mock
  servers rather than trait mocks.
- A dependency that genuinely varies is a generic over its trait (methods
  returning `impl Future + Send`), chosen once in `main` with `run::<S>()`;
  never `Arc<dyn Trait>` and `#[async_trait]`, and never a Cargo feature.
- Routes are registered with aide and a docs function each; the OpenAPI
  document is generated from the body types.
- Handlers classify, call one service method, respond, and report one
  outcome; errors map to a `Problem` body (`Problem::response`) in one
  function per error type.
- `OtelTraceLayer` outermost, `HttpMetricsLayer` inside it; comment any layer
  whose position matters.
- End-to-end tests in `tests/<group>.rs` cover the happy path, each refusal
  (with no dependency called), upstream errors, an unreachable upstream,
  headers and metrics.

## Metrics

Follow OpenTelemetry's semantic conventions, recorded through the `metrics`
facade.

- **Use tower-telemetry's layers, not hand-rolled request metrics.**
  `HttpMetricsLayer` for a server, `HttpClientMetricsLayer` for a client.
  Use the `tower-telemetry` crate and configure it alongside the service's
  exporter. They record the conventions' attributes.
- **Request metrics follow OpenTelemetry semantic conventions.** Record
  `http.server.request.duration` and its standard attributes. `tower-telemetry`
  also records the `http.server.requests` counter because the conventions do
  not define a request counter. Configure duration and body-size histogram
  buckets in the Prometheus exporter setup.
- **Name your own metrics the OpenTelemetry way.** Dotted, lowercase, in a namespace
  (`http.client.request.duration`, `http.client.pool.retries`), with no unit
  or `_total` in the name: describe the unit with `describe_*!(…, Unit::…)`
  and let the exporter add suffixes and turn dots into underscores.
- **Use the conventions' attributes where one exists**: `http.request.method`,
  `http.response.status_code`, `server.address`, `network.protocol.version`,
  `error.type`. Own labels only for what the conventions don't cover, and
  keep them low-cardinality.
- **Give histograms buckets.** Configure duration and body-size buckets with
  `PrometheusBuilder::set_buckets_for_metric` so Prometheus exports histogram
  buckets instead of summaries.
- **Existing names stay until their consumers move.** Rename a metric only
  with the dashboard or collector change that follows.

## Tests

- Unit tests sit in the file they test (`#[cfg(test)] mod tests`) and reach
  private items. A test that drives the crate through its public API against
  a mock server (`httpmock`), a socket or the real router is an integration or
  end-to-end test, and goes in the crate's `tests/` folder
  (`tests/orders.rs`, `tests/client_metrics.rs`).
- Test the rules where they live: a pure function gets exhaustive cases, the
  service gets the paths through it.
- No sleeps to wait for state: poll with a deadline, or assert on what was
  recorded.
