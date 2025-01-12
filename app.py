"""Travel Mapper application to map your travel itinerary from tripit"""

import os
import time
from typing import Any, Dict

from robyn import Request, Robyn
from robyn.logger import Logger
from robyn.openapi import License, OpenAPI, OpenAPIInfo

from sql import DatabaseManager
from tripit import TripIt

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

logger = Logger()
api_url = os.getenv("API_URL") or "https://api.tripit.com"
consumer_key = os.getenv("CONSUMER_KEY")
consumer_secret = os.getenv("CONSUMER_SECRET")

client = TripIt(consumer_key, consumer_secret)
db = DatabaseManager("travel_map.db")


@app.get("/oauth/callback")
async def oauth_callback(request: Request) -> Dict[str, str]:
    oauth_token = request.query_params.get("oauth_token")
    try:
        state = db.get_oauth_state(oauth_token)

        if time.time() - state.timestamp > 1800:
            db.delete_oauth_state(oauth_token)
            return {"error": "OAuth token expired"}, {}, 400

        access_token, access_secret = client.get_access_token(
            state.request_token, state.request_secret
        )

        if db.store_tokens(access_token, access_secret):
            db.delete_oauth_state(oauth_token)
            return {"redirect_to": "/oauth/success"}
        else:
            logger.error("Failed to store tokens")
            return {"error": "Failed to store tokens"}, {}, 500

    except Exception as e:
        logger.error(e)
        return {"error": "Invalid OAuth token"}, {}, 400


@app.get("/oauth/initiate")
async def initiate_oauth():
    request_token, request_secret = client.get_request_token()
    db.store_oauth_state(request_token, request_secret)
    auth_url = client.get_authorization_url(
        request_token, "http://pints.me/oauth/callback"
    )
    return {"redirect_to": auth_url}


@app.get("/api/trips")
async def get_trips(request):
    access_token = request.headers.get("Authorization")
    if not access_token:
        return {"error": "Unauthorized"}, {}, 401

    try:
        tokens = db.get_tokens(access_token)
        if not tokens:
            return {"error": "Invalid token"}, {}, 401

        response = client.list_trip(tokens[0], tokens[1])
        return response

    except Exception as e:
        return {"error": str(e)}, {}, 500


if __name__ == "__main__":
    app.start(host="0.0.0.0", port=8080)
