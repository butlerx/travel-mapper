"""Travel Mapper application to map your travel itinerary from tripit"""

from os import getenv

from opentelemetry import propagate, trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor, ConsoleSpanExporter
from opentelemetry.trace import set_span_in_context
from opentelemetry.trace.propagation.tracecontext import TraceContextTextMapPropagator
from robyn import Request, Response, Robyn, jsonify
from robyn.exceptions import HTTPException
from robyn.logger import Logger
from robyn.openapi import License, OpenAPI, OpenAPIInfo

from middlewares import JWTCookieHandler
from routes import setup_routes
from templates import templates
from tripit import TripIt

trace.set_tracer_provider(TracerProvider())
trace.get_tracer_provider().add_span_processor(
    BatchSpanProcessor(ConsoleSpanExporter())
)

logger = Logger()
tracer = trace.get_tracer(__name__)

app = Robyn(
    file_object=__file__,
    openapi=OpenAPI(
        info=OpenAPIInfo(
            title="Travel Mapper",
            description="Application to map your travel itinerary from tripit",
            version="1.0.0",
            license=License(
                name="BSD2.0",
                url="https://opensource.org/license/bsd-2-clause",
            ),
        ),
    ),
)


def setup_dependencies(app: Robyn) -> None:
    api_url = getenv("API_URL") or "https://api.tripit.com"
    consumer_key = getenv("CONSUMER_KEY")
    consumer_secret = getenv("CONSUMER_SECRET")
    if not consumer_key or not consumer_secret:
        raise ValueError("Missing consumer key or secret environment variables")

    client = TripIt(consumer_key, consumer_secret)
    app.inject_global(tripit_client=client)


@app.before_request()
def extract_context(request: Request) -> Request:
    carrier = {"traceparent": request.headers.get("Traceparent")}
    ctx = TraceContextTextMapPropagator().extract(carrier=carrier)
    span = tracer.start_span(
        f"{request.method} {request.url}",
        context=ctx,
        kind=trace.SpanKind.SERVER,
    )
    token = set_span_in_context(span)
    return request


@app.after_request()
def inject_context(response: Response) -> Response:
    span = trace.get_current_span()
    span.set_attribute("http.status_code", response.status_code)
    propagate.inject(response.headers)
    span.end()
    return response


@app.exception
def handle_exception(error: HTTPException) -> Response:
    details = error.detail if hasattr(error, "detail") else str(error)
    status_code = error.status_code if hasattr(error, "status_code") else 500
    span = trace.get_current_span()
    span.record_exception(error)
    span.set_status(trace.Status(trace.StatusCode.ERROR, details))
    return Response(
        status_code=status_code,
        description=jsonify({"success": False, "error": details}),
        headers={"content-type": "application/json"},
    )


@app.get("/")
def redirect(request: Request):
    return Response(
        status_code=307,
        description="Redirecting to login",
        headers={"Location": "/user/login?error=Please login first"},
    )


@app.get("/dashboard", auth_required=True)
def dashboard(request: Request):
    return templates.render_template("dashboard.html", **request.identity.claims)


setup_routes(app)
setup_dependencies(app)
app.configure_authentication(JWTCookieHandler())


if __name__ == "__main__":
    app.start(host="0.0.0.0", port=8080)
