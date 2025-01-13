"""
  Methods to interact with the TripIt v1 API
"""

import secrets
import time
from typing import Any, Dict, Optional, Tuple

import aiohttp
import oauthlib.oauth1


class TripIt:
    OAUTH_SIGNATURE_METHOD = "HMAC-SHA1"

    def __init__(
        self,
        consumer_key: str,
        consumer_secret: str,
        api_url: str = "https://api.tripit.com",
    ):
        if not consumer_key or not consumer_secret:
            raise ValueError("Consumer key and secret are required")
        self.consumer_key = consumer_key
        self.consumer_secret = consumer_secret
        self.base_url = api_url
        self._api_version = "v1"

    async def get_request_token(self) -> Tuple[str, str]:
        auth = oauthlib.oauth1.Client(
            self.consumer_key,
            client_secret=self.consumer_secret,
            signature_method=TripIt.OAUTH_SIGNATURE_METHOD,
            timestamp=str(int(time.time())),
            nonce=secrets.token_hex(40),
        )
        url = f"{self.base_url}/oauth/request_token"
        uri, signed_headers, signed_body = oauth.sign(url, http_method="POST")

        async with aiohttp.ClientSession() as session:
            async with session.post(uri, headers=signed_headers) as response:
                if response.status_code != 200:
                    raise Exception("Failed to get request token")
                credentials = dict(x.split("=") for x in response.text.split("&"))
                return credentials["oauth_token"], credentials["oauth_token_secret"]

    def get_authorization_url(self, request_token: str, callback_url: str) -> str:
        return f"https://www.tripit.com/oauth/authorize?oauth_token={request_token}&oauth_callback={callback_url}"

    async def get_access_token(
        self, request_token: str, request_token_secret: str
    ) -> Tuple[str, str]:
        auth = oauthlib.oauth1.Client(
            self.consumer_key,
            client_secret=self.consumer_secret,
            resource_owner_key=request_token,
            resource_owner_secret=request_token_secret,
            signature_method=TripIt.OAUTH_SIGNATURE_METHOD,
            timestamp=str(int(time.time())),
            nonce=secrets.token_hex(40),
        )
        url = f"{self.base_url}/oauth/access_token"
        uri, signed_headers, signed_body = oauth.sign(url, http_method="POST")

        async with aiohttp.ClientSession() as session:
            async with session.post(uri, headers=signed_headers) as response:
                if response.status_code != 200:
                    raise Exception("Failed to get request token")
                credentials = dict(x.split("=") for x in response.text.split("&"))
                return credentials["oauth_token"], credentials["oauth_token_secret"]

    async def _do_request(
        self,
        access_token: str,
        access_secret: str,
        endpoint: str,
        method="GET",
        params: Optional[Dict[str, str]] = None,
        data: Optional[Dict[str, Any]] = None,
    ):
        auth = oauthlib.oauth1.Client(
            client_key=self.consumer_key,
            client_secret=self.consumer_secret,
            resource_owner_key=access_token,
            resource_owner_secret=access_secret,
            signature_method=TripIt.OAUTH_SIGNATURE_METHOD,
        )
        async with aiohttp.ClientSession() as session:
            params = params or {}
            params["format"] = "json"
            url = f"{self.base_url}/{self._api_version}/{endpoint}"
            json = data if data else None

            uri, signed_headers, signed_body = oauth.sign(
                url, http_method=method, body=json, parameters=params
            )

            async with session.request(
                method, uri, headers=signed_headers, data=signed_body
            ) as response:
                response.raise_for_status()
                return await response.json()

    async def get_trip(
        self,
        access_token: str,
        access_secret: str,
        id: str,
        filters: Optional[Dict[str, str]] = None,
    ):
        params = filters or {}
        params["id"] = id
        return await self._do_request(
            access_token, access_secret, f"get/trip/id/{id}", params=params
        )

    async def get_air(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(access_token, access_secret, f"get/air/id/{id}")

    async def get_lodging(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"get/lodging/id/{id}"
        )

    async def get_car(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(access_token, access_secret, f"get/car/id/{id}")

    async def get_rail(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(access_token, access_secret, f"get/rail/id/{id}")

    async def get_transport(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"get/transport/id/{id}"
        )

    async def get_cruise(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"get/cruise/id/{id}"
        )

    async def get_restaurant(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"get/restaurant/id/{id}"
        )

    async def get_activity(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"get/activity/id/{id}"
        )

    async def get_profile(self, access_token: str, access_secret: str):
        return await self._do_request(access_token, access_secret, "get/profile")

    async def list_trip(
        self,
        access_token: str,
        access_secret: str,
        filters: Optional[Dict[str, str]] = None,
    ):
        return await self._do_request(
            access_token, access_secret, "list/trip", params=filters
        )

    async def create(self, access_token: str, access_secret: str, data: Dict[str, Any]):
        return await self._do_request(
            access_token, access_secret, "create", method="POST", data=data
        )

    async def replace_trip(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/trip/id/{id}",
            method="POST",
            data=data,
        )

    async def delete_trip(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/trip/id/{id}", method="POST"
        )

    async def get_points_program(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"get/points_program/id/{id}"
        )

    async def get_note(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(access_token, access_secret, f"get/note/id/{id}")

    async def get_map(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(access_token, access_secret, f"get/map/id/{id}")

    async def get_directions(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"get/directions/id/{id}"
        )

    async def delete_air(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/air/id/{id}", method="POST"
        )

    async def delete_lodging(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/lodging/id/{id}", method="POST"
        )

    async def delete_car(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/car/id/{id}", method="POST"
        )

    async def delete_rail(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/rail/id/{id}", method="POST"
        )

    async def delete_transport(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/transport/id/{id}", method="POST"
        )

    async def delete_cruise(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/cruise/id/{id}", method="POST"
        )

    async def delete_restaurant(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/restaurant/id/{id}", method="POST"
        )

    async def delete_activity(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/activity/id/{id}", method="POST"
        )

    async def delete_note(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/note/id/{id}", method="POST"
        )

    async def delete_map(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/map/id/{id}", method="POST"
        )

    async def delete_directions(self, access_token: str, access_secret: str, id: str):
        return await self._do_request(
            access_token, access_secret, f"delete/directions/id/{id}", method="POST"
        )

    async def replace_air(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/air/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_lodging(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/lodging/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_car(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/car/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_rail(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/rail/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_transport(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/transport/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_cruise(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/cruise/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_restaurant(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/restaurant/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_activity(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/activity/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_note(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/note/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_map(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/map/id/{id}",
            method="POST",
            data=data,
        )

    async def replace_directions(
        self, access_token: str, access_secret: str, id: str, data: Dict
    ):
        return await self._do_request(
            access_token,
            access_secret,
            f"replace/directions/id/{id}",
            method="POST",
            data=data,
        )

    async def list_object(
        self,
        access_token: str,
        access_secret: str,
        filters: Optional[Dict[str, str]] = None,
    ):
        return await self._do_request(
            access_token, access_secret, "list/object", params=filters
        )

    async def list_points_program(self, access_token: str, access_secret: str):
        return await self._do_request(
            access_token, access_secret, "list/points_program"
        )

    async def crs_load_reservations(
        self,
        access_token: str,
        access_secret: str,
        data: Dict[str, Any],
        company_key: Optional[str] = None,
    ):
        params = {"company_key": company_key} if company_key else None
        return await self._do_request(
            access_token,
            access_secret,
            "crs/load_reservations",
            method="POST",
            params=params,
            data=data,
        )

    async def crs_delete_reservations(
        self, access_token: str, access_secret: str, record_locator: str
    ):
        return await self._do_request(
            access_token,
            access_secret,
            "crs/delete_reservations",
            method="POST",
            params={"record_locator": record_locator},
        )
