from time import time

from robyn import Request, SubRouter
from robyn.logger import Logger
from robyn.robyn import QueryParams
from robyn.types import JSONResponse

from database import database_manager
from errors import ExpiredOAuthToken, InternalServerError, InvalidOAuthToken

logger = Logger()
oauth_router: SubRouter = SubRouter(__name__, prefix="/oauth")


class InitiateResponse(JSONResponse):
    redirect_to: str


@oauth_router.get("/initiate", auth_required=True)
async def initiate_oauth(r: Request, global_dependencies) -> InitiateResponse:
    client = global_dependencies["tripit_client"]
    request_token, request_secret = await client.get_request_token()
    with database_manager() as db:
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
    with database_manager() as db:
        state = db.get_oauth_state(oauth_token)

        if time() - state.timestamp > 1800:
            db.delete_oauth_state(oauth_token)
            raise ExpiredOAuthToken()

    try:
        access_token, access_secret = await client.get_access_token(
            state.request_token, state.request_secret
        )
    except Exception as e:
        logger.error("Failed to get Access token: %s" % str(e))
        raise InvalidOAuthToken()

    try:
        with database_manager() as db:
            db.store_tokens(access_token, access_secret)
            db.delete_oauth_state(oauth_token)
        return {"redirect_to": "/oauth/success"}
    except Exception as e:
        logger.error("Failed to store tokens: %s" % str(e))
        raise InternalServerError()
