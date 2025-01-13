from typing import Any, Dict, Tuple

from robyn import Request, SubRouter
from robyn.logger import Logger

from ..errors import InternalServerError, InvalidOAuthToken, Unauthorized

logger = Logger()
trips_router: SubRouter = SubRouter(__name__, prefix="/api/trips")


@trips_router.get("/")
async def get_trips(
    request: Request, global_dependencies
) -> Dict[str, Any] | Tuple[Dict[str, Any], Dict[str, Any], int]:
    access_token = request.headers.get("Authorization")
    if not access_token:
        raise Unauthorized()

    db = global_dependencies["db"]
    client = global_dependencies["tripit_client"]
    try:
        tokens = db.get_tokens(access_token)
        if not tokens:
            raise InvalidOAuthToken()

        response = await client.list_trip(tokens[0], tokens[1])
        return response

    except Exception as e:
        logger.error("Failed to get trips: %s", e)
        raise InternalServerError()
