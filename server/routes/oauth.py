from time import time

from robyn import Request, SubRouter
from robyn.logger import Logger
from robyn.robyn import QueryParams
from robyn.types import JSONResponse

from ..errors import ExpiredOAuthToken, InternalServerError, InvalidOAuthToken

logger = Logger()
oauth_router: SubRouter = SubRouter(__name__, prefix="/oauth")


class InitiateResponse(JSONResponse):
    redirect_to: str


@oauth_router.get("/initiate")
async def initiate_oauth(r: Request, global_dependencies) -> InitiateResponse:
    client = global_dependencies["tripit_client"]
    db = global_dependencies["db"]
    request_token, request_secret = client.get_request_token()
    db.store_oauth_state(request_token, request_secret)
    auth_url = client.get_authorization_url(
        request_token, "http://pints.me/oauth/callback"
    )
    return {"redirect_to": auth_url}


class CallbackRequestParams(QueryParams):
    oauth_token: str


class CallbackResponse(JSONResponse):
    redirect_to: str


@oauth_router.get("/callback")
async def oauth_callback(
    r: Request, query_params: CallbackRequestParams, global_dependencies
) -> CallbackResponse:
    """Handle OAuth callback from TripIt"""
    oauth_token = query_params.get("oauth_token")
    client = global_dependencies["tripit_client"]
    db = global_dependencies["db"]
    state = db.get_oauth_state(oauth_token)

    if time() - state.timestamp > 1800:
        db.delete_oauth_state(oauth_token)
        raise ExpiredOAuthToken()

    try:
        access_token, access_secret = client.get_access_token(
            state.request_token, state.request_secret
        )

        if db.store_tokens(access_token, access_secret):
            db.delete_oauth_state(oauth_token)
            return {"redirect_to": "/oauth/success"}

        logger.error("Failed to store tokens")
        raise InternalServerError()

    except Exception as e:
        logger.error(str(e))
        raise InvalidOAuthToken()
