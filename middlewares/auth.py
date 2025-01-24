from typing import Optional

from robyn.authentication import AuthenticationHandler, Identity, TokenGetter
from robyn.robyn import Request, Response

from database import database_manager, decode_access_token


class CookieGetter(TokenGetter):
    """
    This class is used to get/set the authentication token from/to cookies.
    """

    COOKIE_NAME = "auth_token"

    @classmethod
    def get_token(cls, request: Request) -> Optional[str]:
        """Gets the token from request cookies"""
        cookies = request.headers.get("Cookie")
        if not cookies:
            return None
        for c in cookies.split(";"):
            cookie_name, cookie_value = c.split("=")
            print(cookie_name, cookie_value)
            if cls.COOKIE_NAME == cookie_name.strip():
                return cookie_value.strip()
        return None

    def set_token(cls, request: Request, token: str):
        """Sets the token in request cookies"""
        response = Response(status_code=200, description="OK", headers={})
        response.set_cookie(
            cls.COOKIE_NAME,
            token,
        )
        return response


class JWTCookieHandler(AuthenticationHandler):
    """JWT authentication handler using cookies"""

    def __init__(self):
        super().__init__(CookieGetter())

    def authenticate(self, request: Request) -> Optional[Identity]:
        """Authenticates the request using the cookie token"""
        token = self.token_getter.get_token(request)
        print(token)
        if not token:
            return None

        try:
            payload = decode_access_token(token)
            user_id = payload["sub"]
            print(user_id)
        except jwt.ExpiredSignatureError:
            return None
        except jwt.InvalidTokenError:
            return None

        with database_manager() as db:
            user = db.get_user(user_id)

        claims = {
            "username": user.username,
            "id": str(user.id),
        }
        if user.access_token and user.access_secret:
            claims["access_token"] = user.access_token
            claims["access_secret"] = user.access_secret

        return Identity(claims=claims)
