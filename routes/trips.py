from typing import Any, Dict, Tuple

from robyn import Request, SubRouter
from robyn.logger import Logger

from database import database_manager
from errors import InternalServerError, InvalidOAuthToken, TripItError, Unauthorized

logger = Logger()
trips_router: SubRouter = SubRouter(__name__, prefix="/api/trips")


@trips_router.get("/", auth_required=True)
async def get_trips(
    request: Request, global_dependencies
) -> Dict[str, Any] | Tuple[Dict[str, Any], Dict[str, Any], int]:
    client = global_dependencies["tripit_client"]
    access_token = request.identity.claims.get("access_token")
    if not access_token:
        raise TripItError("Access token not setup please auth with tripit")
    try:
        with database_manager() as db:
            tokens = db.get_tokens(access_token)
        if not tokens:
            raise InvalidOAuthToken()

        response = await client.list_trip(tokens[0], tokens[1])
        return response

    except Exception as e:
        logger.error("Failed to get trips: %s", e)
        raise InternalServerError()
