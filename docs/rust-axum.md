# Axum Server Style Guide

How to build and structure an axum HTTP service (axum 0.8, aide 0.15,
tower 0.5). The general Rust rules in [`rust-style.md`](rust-style.md) apply
throughout; this covers what is specific to an axum server. Examples use a
made-up `orders` service: it stores orders and charges them through a
payments API.

Each section has two parts:

- **Do**: the pattern to copy.
- **Avoid**: a pattern that looks reasonable and that we don't want.

---

## 1. Crate layout

```
src/
  main.rs              # boot only: config, telemetry, state, bind, serve
  lib.rs               # `pub mod` everything so tests/ can import it
  config.rs + config/  # Config: one typed section per concern; loader.rs
  state.rs             # AppState: the services and policies handlers share
  routes.rs            # router(state, &server): merges route groups and surfaces; serve()
  routes/
    problem.rs         #   the service's error body, Problem::response
    openapi.rs         #   the OpenAPI document and /openapi.json
    health.rs          #   GET /healthz
    orders.rs          #   one route group: routes(), handlers, error mapping
    orders/
      docs.rs          #     its OpenAPI operations
      requests.rs      #     its request bodies and query types
      responses.rs     #     its response bodies
  middleware.rs + middleware/  # from_fn functions and the policies they apply
  extractors.rs + extractors/  # custom FromRequestParts / FromRequest, one per file
  services.rs + services/      # what the service does, without HTTP: one per domain
  models.rs + models/          # domain types shared by several services
  clients.rs + clients/        # thin wrappers over a dependency only this crate uses
  metrics.rs           # exporter setup and the /metrics listener
  error.rs             # errors that stop the service
  shutdown.rs          # the shutdown signal
config/
  default.toml         # every setting's default; <ENV>.toml per environment
tests/
  orders.rs            # end-to-end, one file per route group: the real router
                       # against mock servers (tests/it/ once there are several)
```

Create a module when it has something in it. A service whose domain types and
clients live in library crates has no `models/` or `clients/`.

**Dependency direction**

```
routes ──► services ──► models, clients, domain crates
  │
  ├──► extractors ──► state, models
  └──► middleware ──► state
```

Services never import from `routes`, and nothing outside `routes`,
`middleware` and `extractors` imports axum.

**Do**
- Keep `main.rs` to wiring. It loads config, starts telemetry, builds
  `AppState`, binds, and calls `routes::serve`. It returns `ExitCode` and
  reports a config error to stderr, since telemetry isn't up yet.
- Declare modules in `lib.rs` as `pub mod`, and have `main.rs` `use` them, so
  the module tree is declared once and `tests/` compiles against the library.
- Name route groups and services for the resource, in the plural:
  `routes/orders.rs`, `services/orders.rs`, `tests/orders.rs`.
- Move a rule, model or client into a library crate when a second binary needs
  it, or when it's a protocol worth testing exhaustively with no HTTP around
  it.
- `module.rs` + `module/`, never `mod.rs`.
- Declare any custom cfg (`cfg(local)`, `cfg(coverage)`) with `check-cfg` in
  the crate's `[lints.rust]`, so a typo in one warns instead of silently
  compiling the code out.

**Avoid**
- Declaring the module tree twice, once in `main.rs` and once in `lib.rs`.
- One file per layer for the whole service (`handlers.rs`, `services.rs`
  holding everything). Split by resource inside each layer.
- Empty `models/` or `clients/` modules created for symmetry.

---

## 2. Startup and configuration

**Do**
- Load configuration once, in layers: `default.toml`, `<ENV>.toml`,
  `overrides.toml`, then `SECTION__KEY` environment variables
  (`config/loader.rs`, with the `config` crate). Keys are kebab-case; each
  section is a typed struct with `#[serde(default, rename_all = "kebab-case")]`,
  in its own file under `src/config/`.
- Define each default once, as a function named for the field, and point
  both serde and `Default` at it with `derivative`, so a missing key and
  `ServerConfig::default()` can't disagree:

  ```rust
  #[derive(Clone, Deserialize, Derivative)]
  #[derivative(Debug, Default)]
  #[serde(default, rename_all = "kebab-case")]
  pub struct ServerConfig {
      #[derivative(Default(value = "default_port()"))]
      #[serde(default = "default_port")]
      pub port: u16,
      /// How long in-flight requests may drain after a shutdown signal.
      #[derivative(Default(value = "default_shutdown_timeout()"))]
      #[serde(default = "default_shutdown_timeout", with = "humantime_serde")]
      pub shutdown_timeout: Duration,
  }

  const fn default_port() -> u16 {
      8080
  }

  const fn default_shutdown_timeout() -> Duration {
      Duration::from_secs(25)
  }
  ```

  Durations and sizes are human-readable strings: `"25s"` through
  `humantime_serde` (`Duration`), `"2MiB"` through `bytesize`'s `serde`
  feature (`ByteSize`). Never a bare number whose unit only a comment
  names.
  A field whose type's own `Default` is right needs neither attribute.
- Validate what has no usable default at load time (`Config::validate`) and
  exit non-zero. A task that can't boot is better than one that 500s.
- Build clients once in `main`, hand them to the services that use them, and
  move the services into `AppState`. Pass each component the config section
  it needs, in its constructor.
- Shut down gracefully on `SIGTERM` (how ECS stops a task) and `SIGINT`:
  `axum::serve(listener, app).with_graceful_shutdown(signal)`. Share one
  signal between listeners (`shutdown::signal().shared()`) so the app and the
  metrics endpoint stop together and in-flight requests drain.
  `tokio::signal::ctrl_c()` alone only catches `SIGINT`.
- Bound the drain with a timeout from config, so one stuck connection can't
  hold the process past the orchestrator's kill deadline: `server.shutdown-timeout`
  defaults to 25 seconds, inside ECS's default 30-second stop timeout.

**Avoid**
- `std::env::var` scattered through startup. Read config into structs first,
  then wire clients. The one exception is a fallback the shared config can't
  express (`SENTRY_DSN`), and it lives next to the code that needs it.
- Returning many values as a tuple from a config helper. Use a named struct.
- `expect()` on config at startup. Return a `ConfigError` and let `main`
  report it, so the message is a sentence, not a panic.
- A global `static CONFIG` that layers and services read directly. It's a
  hidden dependency that tests can't vary. Pass config in.
- A hand-rolled accept loop with `Ok(conn) = listener.accept()` inside
  `select!`: one accept error (`EMFILE`) disables that branch until shutdown.
  Use `axum::serve`, or handle the `Err` arm.

---

## 3. Application state

**Do**
- One `#[derive(Clone, Debug)] AppState` in `state.rs`, holding the services
  and the policies middleware applies. `router()` wraps it in `Arc` once, so
  per-request clones are a refcount. Keep the fields private, build it with
  `new`, and hand out borrows, not clones:

  ```rust
  #[derive(Debug, Clone)]
  pub struct AppState {
      orders: OrderService,
      cors: Cors,
  }

  impl AppState {
      #[must_use]
      #[inline]
      pub fn orders(&self) -> &OrderService {
          &self.orders
      }
  }
  ```
- Prefer concrete, cheaply cloneable services that own clients sharing their
  connection pools. Tests then build the real service pointed at a mock
  server (section 13).
- When a dependency genuinely varies (a backend chosen by config), make the
  state generic over its trait (`AppState<S: Store>`) and pick the
  implementation once, in `main`, by calling a generic `run::<S>()`. Calls
  are dispatched statically and can be inlined, and traits that return
  `impl Future` or have associated types work as they are. The price is
  `<S>` on the handlers and router functions, and one compiled copy per
  implementation.
- Implement `FromRef<Arc<AppState>>` for a service or policy when a middleware
  or extractor needs only that piece: `State<OrderService>` reads better
  than reaching through the whole state. Inject only what a component needs.
- Once state has more than a handful of fields, give it a constructor
  (`AppState::from_config(&config).await`) so `main` and the test harness
  build it the same way.
- Write a hand-written `Debug` for anything holding secrets (a client holding
  an API key redacts it).

**Avoid**
- Clients in `AppState` that handlers call directly. Handlers call services;
  services call clients.
- `Arc<dyn Trait>` for a dependency on the hot path. A trait whose methods are
  `async` isn't dyn-compatible without `#[async_trait]`, which boxes the
  future of every call.
- `Arc<dyn Trait>` fields added only so tests can swap in mocks. Test the
  real client against `httpmock` instead (section 13).
- A Cargo feature that picks an implementation at build time. A build can
  then run only one, and tests against the other can't run in the build that
  ships. This server uses Prometheus for metrics and OTLP/gRPC for traces,
  configured with `METRICS_PROMETHEUS_ENDPOINT` and
  `OTEL_EXPORTER_OTLP_ENDPOINT`.

---

## 4. Routing

**Do**
- Give each route group a `routes() -> ApiRouter<Arc<AppState>>` with its
  paths, handlers and OpenAPI operations, and let `routes.rs` compose them:

  ```rust
  pub fn router(state: AppState, server: &ServerConfig) -> Router {
      let state = Arc::new(state);
      let mut api = openapi::document();
      ApiRouter::new()
          .merge(health::routes())
          .merge(orders::routes())
          .route("/openapi.json", get(openapi::serve))
          .finish_api(&mut api)
          .fallback(problem::not_found)
          .method_not_allowed_fallback(problem::method_not_allowed)
          // …the shared middleware, then the outer shell (section 5), its
          // limits from `server`
          .with_state(state)
  }
  ```

  Route groups return state-less routers; `router()` applies the state once.
- When groups need different middleware (a public API with auth and rate
  limits, an internal API, webhooks), group them into **surfaces**: one
  function per surface that merges its route groups and adds that surface's
  layers, merged into the router like a group.
- Per-route concerns go on `route_layer` (a required permission, a feature
  gate); per-surface concerns on `layer` (auth, rate limits, body limits,
  CORS).
- Build the router in `router(state, &config.server) -> Router`, separate
  from `serve(listener, app, shutdown, drain)`, so tests drive it without a socket
  and with their own limits.
- Register documented routes with aide's `api_route`, pairing each handler
  with its docs function (`post_with(create_order, docs::create_order)`).
- Use `{param}` path syntax, and `{*rest}` for a wildcard; axum 0.8 panics
  at startup on the old `:param` and `*rest` forms. Name segments for what
  they hold: `/accounts/{account}/orders/{order_id}`; resource paths are
  plural nouns.
- Answer unmatched paths and methods with the service's error body: route
  the fallback and the method-not-allowed fallback to handlers that return a
  `Problem`. When a protocol owns its error answers, route them to the
  protocol's handler instead.
- Comment any layer whose position matters. The last `.layer()` added runs
  first on the request:

  ```rust
  .layer(HttpMetricsLayer::default()) // Record `http.server.request.duration` per OTel conventions.
  // Outside every layer that answers on its own, so a panic's 500, a
  // timeout's 408 and a 413 carry the request ID too.
  .layer(PropagateRequestIdLayer::x_request_id())
  .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
  // Outermost last: the span covers everything above.
  .layer(OtelTraceLayer::<PropagatingSpanDecorator, HealthcheckFilter>::default())
  ```

**Avoid**
- One `router()` listing every route in the service. Each group owns its
  routes.
- Buffering request bodies for an endpoint that streams them. A streaming
  handler takes the raw `Request` and passes the body on frame by frame;
  `DefaultBodyLimit` and `Bytes` extractors don't belong on that route.
- Boolean arguments that configure a router or its middleware. Use a struct
  field with a name.

---

## 5. Middleware

| Need | Use | Example |
| --- | --- | --- |
| Request-scoped logic local to this crate | `axum::middleware::from_fn` / `from_fn_with_state`, in `middleware/` | headers every response gets |
| Reusable across services | A `tower::Layer` in a library crate | tower-telemetry's `OtelTraceLayer`, `HttpMetricsLayer` |
| Hand data from a middleware to a handler | Insert into the request's extensions; read it with an extractor (section 6) | the caller an auth middleware resolved |
| Hand data from a handler to a layer | Insert into the response's extensions | `MetricLabels` for `HttpMetricsLayer` |

**Do**
- Default to `from_fn`. Write a tower `Service` only when it must be generic
  over body types or shared outside the crate, and then put it in a library
  crate.
- Keep the policy a middleware applies as a type beside it (`Cors` in
  `middleware/cors.rs`), held in `AppState`, so a handler that answers part
  of it (a CORS preflight) uses the same rules.
- Give each surface's standard stack one function, so every route group in
  it gets the same layers in the same order.
- Give every surface the same outer shell, outermost first: trace → request
  ID → request metrics → error bodies → body limit → catch panic → timeout.
  The request ID goes outside everything that can answer on its own, so a
  panic's `500`, a timeout's `408` and a `413` carry it too. Auth and domain
  layers go inside the shell. tower-http has `SetRequestIdLayer`,
  `PropagateRequestIdLayer`, `RequestBodyLimitLayer`, `CatchPanicLayer` and
  `TimeoutLayer`. A route that streams large bodies leaves out the body limit
  and sets its own timeout.
- Turn the responses those layers make on their own (a panic's `500`, a
  timeout's `408`, a `413`) into the `Problem` body, so clients see one
  error format: `CatchPanicLayer::custom` for the panic, and a `from_fn`
  middleware outside the limit and timeout layers for the rest.
- Set a timeout on every surface. A request with no deadline holds a
  connection and a task forever.
- Pass a fact one layer established to the layers inside it as a
  purpose-built marker type in the request's extensions
  (`struct Authenticated;`), so nothing checks it twice.
- Unit-test each layer alone:
  `ServiceBuilder::new().layer(X).service(service_fn(...))`, asserting on
  what reached the inner service and what came back.
- Apply headers every response needs (`Server`, `Cache-Control`, CORS) in one
  middleware, so error responses and the health check get them too.

**Avoid**
- Setting the same response header in each handler.
- Building the same policy (a `CorsLayer`, an auth check) in several places.

---

## 6. Extractors

axum's [`extract` docs](https://docs.rs/axum/latest/axum/extract/index.html)
cover the built-in extractors, the order they must come in, and how to write
one. A custom extractor looks like this:

```rust
// extractors/api_key.rs
//! [`ApiKey`]: the caller's `X-Api-Key`.

use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};
use http::{HeaderName, StatusCode, request::Parts};

use crate::routes::problem::Problem;

static X_API_KEY: HeaderName = HeaderName::from_static("x-api-key");

/// The caller's API key, from `X-Api-Key`. No `Debug`: it's a secret.
pub struct ApiKey(pub String);

/// `X-Api-Key` was missing, empty or not visible ASCII.
pub struct MissingApiKey;

impl IntoResponse for MissingApiKey {
    fn into_response(self) -> Response {
        Problem::response(StatusCode::UNAUTHORIZED, "missing-api-key", "X-Api-Key is required")
    }
}

impl<S: Send + Sync> FromRequestParts<S> for ApiKey {
    type Rejection = MissingApiKey;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(&X_API_KEY)
            .and_then(|value| value.to_str().ok())
            .filter(|key| !key.is_empty())
            .map(|key| Self(key.to_owned()))
            .ok_or(MissingApiKey)
    }
}

// Adds no parameters of its own: the route's docs function names the header
// and its 401.
impl aide::OperationInput for ApiKey {}
```

A handler then takes it like any other extractor, after `State` and the path:
`async fn get_order(State(state): State<Arc<AppState>>, Path(id): Path<String>, key: ApiKey)`.

**Do**
- One extractor per file in `extractors/`, re-exported from `extractors.rs`.
- Prefer `FromRequestParts`: it doesn't consume the body, so it composes with
  any body extractor. Use `FromRequest` only for the body.
- When the handler needs the request whole (a protocol that reads the raw
  path, query and headers itself), take axum's `Request` directly. It is
  already an extractor, and aide documents it as an input; describe its
  parameters in the docs function.
- Give every extractor a rejection that implements `IntoResponse` through
  `Problem::response`, so a failed extraction answers with the service's
  error body.
- When middleware inserts an extension for handlers, add an extractor that
  reads it, and make a missing extension a programming error that names the
  fix:

  ```rust
  .expect("Caller is missing: is the auth middleware applied to this route?")
  ```
- Implement `aide::OperationInput` for the extractor, describing what it
  reads, so the OpenAPI document stays complete. An empty impl is enough
  when it adds no parameters.
- Write extractors without `#[async_trait]`: in axum 0.8 `FromRequestParts`
  and `FromRequest` are plain traits with an `async fn`, as above.
- Implement `OptionalFromRequestParts` when `Option<T>` should mean "absent
  is fine". In axum 0.8 `Option<T>` no longer swallows `T`'s rejection: a
  present but invalid value is still refused.
- Accept a JSON body through a `ValidatedJson<T>` extractor, not `Json<T>`.
  It checks `Content-Type`, deserialises with `serde_path_to_error` so the
  error points at the bad field, converts into checked types (`TryFrom`), and
  answers with `Problem::response` on any failure. Its `OperationInput`
  delegates to `Json<T>`, so the body stays in the document.

**Avoid**
- Parsing headers or query values by hand in several handlers. Write the
  extractor once.
- A newtype around an existing extractor that only adds an empty
  `OperationInput` impl. Check aide first: it implements `OperationInput` for
  `Request`, `Body`, `Parts`, `State`, `Extension`, `Path`, `Query`, `Json` and
  more.
- axum's built-in `Json<T>` and `Query<T>` on public endpoints without
  handling their rejections: they answer in plain text, not our error body.
  Wrap them, or map their rejection.

---

## 7. Handlers

**Do**
- Keep handlers thin. Four steps:
  1. Extract and classify the request (an extractor, or a pure function in
     the domain crate or `models/`).
  2. Call one service method.
  3. Turn the service's outcome into a response
     (`fn respond(OrderOutcome) -> (Response, Outcome)`).
  4. Report the outcome once.
- Give every way a request can end a variant of one enum
  (`Outcome::Created`, `Outcome::Refused`, `Outcome::PaymentsUnreachable`, …)
  with `label() -> &'static str`, and report it in one place: the outcome
  counter and the span attribute.
- Name handlers `<verb>_<resource>` (`get_order`, `create_order`), one
  convention per crate; a handler that serves every method of a protocol is
  named for the protocol.
- Return concrete types that aide can document: `Json<T>`, `Response` for a
  protocol handler documented by its docs function, or
  `Result<Json<T>, E>` where `E: IntoResponse + aide::OperationOutput`, so
  `?` works. Map each service error to a response in one function in the
  route group (section 10).
- Order extractors `State` → `Path` / `Query` → custom part extractors → the
  body extractor last.

**Avoid**
- Business rules in handlers: deciding what a request means, calling two
  dependencies in sequence, choosing what to write where. Those go in a
  service, or in a domain crate when they are pure.
- `match` on a result in every handler just to call `.into_response()` on
  the error. Implement `IntoResponse` for the error and use `?`.
- `-> impl IntoResponse` on a documented handler. aide can't see through it,
  so the response drops out of the OpenAPI document.

---

## 8. Services

**Do**
- One service per domain in `services/`, a struct holding the clients it
  needs (`OrderService { store: Store, payments: PaymentsClient }`),
  constructed in `main` and held in `AppState`.
- Take domain inputs (checked types like `OrderId`, a `NewOrder` struct) and
  return a domain outcome: an enum of every way the operation can end
  (`OrderOutcome::Created`, `::Refused`, `::PaymentFailed`, …), or
  `Result<T, E>` with a `thiserror` enum per operation.
- No axum types. `http` types are fine where they are the data (an upstream's
  `StatusCode` passed on), and bodies are any `http_body::Body`, so the
  service works with whatever the route hands it.
- Record what only the service knows: dependency metrics
  (`upstream_errors_total` for a failed payments call), and logs of domain
  events (`order created`) with structured fields.
- Put `#[tracing::instrument(skip_all, fields(…))]` on a service method worth
  its own span; `skip_all` keeps bodies and secrets out of traces, and
  `fields` names what to record.
- Build each upstream client once and share its connection pool, with a
  connect timeout and a request timeout.
- Retry only idempotent upstream calls, or calls that failed before anything
  was sent (a connection failure), with exponential backoff and jitter.
- Deduplicate concurrent identical expensive work (a token fetch, a
  certificate mint) with a single-flight guard, and refresh cached values
  before they expire, not on the request that finds them stale.
- Prefer concrete services. When a dependency genuinely varies (a storage
  backend chosen by config), write a trait for it and make the service
  generic over it:

  ```rust
  pub trait OrderRepository: Send + Sync + 'static {
      fn get(&self, id: &OrderId) -> impl Future<Output = Result<Option<Order>, RepositoryError>> + Send;
      fn insert(&self, order: &Order) -> impl Future<Output = Result<(), RepositoryError>> + Send;
  }

  #[derive(Clone)]
  pub struct OrderService<R> {
      repository: Arc<R>,
      payments: PaymentsClient,
  }
  ```
- Write the trait's methods as `fn … -> impl Future<Output = …> + Send`, so
  callers can spawn their futures. No `#[async_trait]`: it boxes the future
  of every call.
- Pick the implementation once, in `main`, by calling a generic
  `run::<R>()`; the state, handlers and routers carry `<R>`. Calls are
  dispatched statically and can be inlined, which `Arc<dyn Trait>` can't do.
- Test with a real in-memory implementation of the trait, held to the same
  conformance tests as the production one, rather than a `mockall` mock.
- Split a service into a directory module once it passes a few hundred lines:
  `services/orders.rs` + `services/orders/{create.rs, refund.rs}`.
- Pass a struct when an operation takes more than three or four inputs
  (`NewOrder { account, items, currency, reference }`).

**Avoid**
- A service calling another service's clients directly. Depend on the other
  service, or move the shared part into a client or model.
- `Arc<dyn Trait>` and `#[async_trait]` for a dependency that varies. Every
  call allocates its future, and a trait with associated types or
  `impl Future` returns can't be `dyn` at all.
- Returning `Response`, `Json` or a status the client should get. The route
  decides the response; the service says what happened.
- Long positional argument lists, and `None`s at call sites.

---

## 9. Models and bodies

Three kinds of type, each in its own place:

| Kind | Lives in | Example |
| --- | --- | --- |
| Request and response bodies (the wire format) | `routes/<group>/requests.rs`, `responses.rs` | `CreateOrder`, `OrderResponse` |
| Domain types shared by several services | `models/`, or a domain crate when another binary needs them | `Order`, `OrderId`, `Currency` |
| Database rows and queries | `db/`, per [`rust-sqlx.md`](rust-sqlx.md) | — |

**Do**
- Declare bodies as typed structs with `#[derive(Serialize, JsonSchema)]`
  (and `Deserialize, JsonSchema` for requests), so the OpenAPI document is
  generated from the same type that is sent.
- Name fields as the wire contract does. An existing protocol keeps its
  names; a new API picks one case and states it in its spec.
- Use `&'static str` for fixed values (`status: "ok"`) and `Cow<'static,
  str>` for messages that are usually fixed.
- Parse client input into checked types at the boundary (`OrderId`,
  `Quantity`); code past the boundary takes the type, not a `&str`.
- Convert between models and bodies in the route group (`From`, or the
  `respond` function). Never serialize a model, a client's type or a database
  row directly.

**Avoid**
- `serde_json::json!` for response bodies. It allocates a `Value` tree and
  skips the schema.
- A shared `models/` of bodies for every route. Bodies belong to their route
  group; promote one to `routes/` only when several groups send it (the
  `Problem` error body).

---

## 10. Errors

| Kind | Type | Lives in | Knows about HTTP? |
| --- | --- | --- | --- |
| A request's rules refusing it | `thiserror` enum with `code()` (`Rejection`) | the domain crate or `models/` | No |
| A service operation failing | `thiserror` enum per operation, or an outcome variant (`OrderOutcome::PaymentFailed`) | `services/` | No |
| The service's own error answer | `Problem`, an RFC 9457 problem document (`status`, `code`, `title`, `detail`; `application/problem+json`) | `routes/problem.rs` | Yes |
| Can't start or keep serving | `error::Error` | `error.rs` | No |

**Do**
- Give each refusal a stable `code() -> &'static str` and a `#[error(…)]`
  message, and map it to a status in one function in the route group
  (`orders::rejected`).
- Build every error response through `Problem::response(status, code,
  detail)`. An error type a handler returns implements `IntoResponse`
  through it and `aide::OperationOutput` (`type Inner = Problem`), so the
  document shows its body.
- Pass an upstream's own error answer through unchanged (status, body,
  `Content-Type`, `WWW-Authenticate`) when the client is better served by it.
- For a failure on our side (an unreachable dependency, an unusable answer),
  answer with a fixed, generic `detail` and log the cause with structured
  fields where it is mapped: `tracing::error!` for `5xx`, nothing or `warn!`
  for expected `4xx`. Decide the level and how opaque the answer is once per
  error type, in its mapping function, not at each call site.
- Write `detail` messages that say what to do next: "Content-Length is
  required to send a body", not "bad request".
- Attach the request's context to the error response's extensions, so the
  access log, metrics and trace record the failure even when a layer returns
  early.

**Avoid**
- Leaking a source error's text to the client on a `5xx`.
- Copying the same generic message into several handlers. One constant or
  one builder.
- `String` variants where the source error type is available. Keep it with
  `#[source]` / `#[from]` so the cause chain reaches the logs.

---

## 11. Observability

Compose telemetry with `tracing` and `tracing-subscriber` for logs,
`tracing-opentelemetry` and OTLP for traces, and `tower-telemetry` for HTTP
spans and metrics. Keep exporter setup in the service's telemetry module.

**Do**
- Make `OtelTraceLayer` the outermost layer, with `PropagatingSpanDecorator`
  (continue the caller's `traceparent`, return ours) and `HealthcheckFilter`
  (no spans for health checks). Spans are named by the matched route, not the
  path.
- Record request metrics with `HttpMetricsLayer`, inside the trace layer.
  It records OpenTelemetry HTTP metrics, including
  `http.server.request.duration`, plus its `http.server.requests` counter.
  Attach labels only the handler knows as `MetricLabels` in the response's
  extensions.
- Export traces over OTLP/gRPC and expose Prometheus metrics on a separate
  listener. Bind the scrape listener to localhost unless the deployment
  explicitly provides a protected metrics network.
- Export the process metrics too: the panic counter, Tokio runtime metrics
  and process metrics.
- Set span attributes the handler learns late with
  `OpenTelemetrySpanExt::set_attribute` on the span the layer put in the
  request's extensions (`Span::record` only reaches declared fields).
- Wrap Tower HTTP clients in `OtelClientTraceLayer` and
  `HttpClientMetricsLayer`, which open the client span, inject trace context
  and record `http.client.*`.
- Give every request an ID: one layer takes the caller's `X-Request-Id` or
  generates one, and another echoes it in the response. Comment its position
  (section 5).
- Open a business span with tenant and resource IDs (account, order) around
  the domain work, inside the request span.
- Classify connection-level errors (the client hung up mid-request, a
  malformed request, shutdown) in one function, logged at `info` or `warn`,
  so they don't page anyone.
- Keep debug and trace logs out of release builds: depend on `tracing` with
  `release_max_level_info`.
- Follow the [Metrics](rust-style.md#metrics) section of the style guide for
  names, attributes and buckets.

**Avoid**
- Logging the same failure at every layer it passes through. Log once, where
  it is handled.
- `format!`-ed log messages. Use structured fields.

---

## 12. OpenAPI

**Do**
- Give each route group a `docs.rs` with one aide docs function per
  operation: summary, description, tag, inputs and each response with its
  body type (`.response_with::<400, Json<Problem>, _>(…)`).
- Assemble the document in `routes/openapi.rs` (`document()` for the
  metadata, `serve` for `/openapi.json`); `router()` fills it with
  `finish_api`.
- Enable aide's `axum`, `axum-json` and `axum-query` features, and use the
  schemars version aide depends on (schemars 0.9 for aide 0.15), or derived
  schemas won't satisfy aide's traits.
- Describe a query the handler reads raw with a `Deserialize + JsonSchema`
  struct that exists only for the document
  (`#[expect(dead_code, reason = "describes the query; the handler reads it raw")]`).
- Test that `/openapi.json` lists the routes and their parameters.

**Avoid**
- `.route` for a public endpoint: it works but is left out of the document.
  Keep `.route` for internal paths like `/openapi.json` itself.
- Hand-written JSON for the document, or a docs function that drifts from
  the handler's real responses. When a handler gains a response, its docs
  function gains it in the same change.

---

## 13. Testing

**Do**
- Unit-test rules in the file they live in (`#[cfg(test)] mod tests`): the
  domain crates exhaustively, plus response builders and body shapes.
- End-to-end tests go in `tests/`, one file per route group: build the
  **real** router with `router(state, &server)`, its real services and clients pointed
  at `httpmock` servers (one per dependency), and send requests with
  `tower::ServiceExt::oneshot`. One `harness()` returns the app and the mock
  servers.
- Name tests as sentences of behaviour:
  `refused_orders_never_call_the_payments_api`,
  `a_payments_rejection_returns_its_status`.
- For each route, cover:
  - the happy path, asserting the body and what reached the mocks
  - each refusal, asserting the status, the `code`, and that no dependency
    was called (`mock.calls_async().await == 0`)
  - an upstream error passed through unchanged
  - an unreachable upstream (`502`, generic body)
  - the headers every response carries
  - the metrics and labels it records, read from the exporter's render
- Once a crate has more than one end-to-end file, make them one test binary:
  `tests/it/main.rs` declares a module per route group, so the test code links
  once and builds faster. Shared fixtures (`harness()`, constants) live in
  `main.rs`.
- Pass config in rather than setting environment variables; a test that must
  set one runs under `serial_test::serial`.
- Test that `/openapi.json` lists every route with its parameters, so a route
  registered with `.route` instead of `api_route` is caught.
- No sleeps to wait for state: poll with a deadline, or assert on what was
  recorded.
- For a proxy or protocol service, add an end-to-end suite that replays
  recorded real traffic against the running binary and compares responses.

**Avoid**
- Mocking a service or client trait to test a handler (`mockall`). Where a
  trait exists because the implementation varies, test with its in-memory
  implementation instead. Otherwise it tests the mock; the
  mock server tests the client, the service, the handler and the wire format
  together.
- Asserting on log output. Assert on responses, mock calls and metrics.

---

## 14. Checklist: adding an endpoint

1. **Contract**: add the requirement and scenarios to the service's spec.
2. **Rule**: add the classification or validation as a pure function, in a
   domain crate or `models/`, returning checked types or a refusal with a
   `code()`; unit-test it exhaustively.
3. **Client**: add the upstream call to its client, taking the checked types.
4. **Service**: add the operation to the domain's service, returning an
   outcome enum or a `Result` with a `thiserror` enum.
5. **Route**: add the handler to the route group, mapping the outcome to a
   response with a typed body or `Problem::response`, and an `Outcome`
   variant; register it in the group's `routes()` with a docs function
   listing every response.
6. **Extractor or middleware**, when the endpoint needs something several
   endpoints share (a caller identity, a permission).
7. **Tests**: add end-to-end cases for the six bullets in section 13 to
   `tests/<group>.rs`.
