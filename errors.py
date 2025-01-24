from robyn.exceptions import HTTPException


class TripItError(HTTPException):
    """Base class for TripIt errors"""

    def __init__(self, status_code: int, message: str):
        super().__init__(status_code, message)


class InvalidOAuthToken(HTTPException):
    """The OAuth token retrived is invalid"""

    def __init__(self):
        super().__init__(400, "Invalid OAuth token")


class ExpiredOAuthToken(HTTPException):
    """The OAuth token has expired"""

    def __init__(self):
        super().__init__(401, "Expired OAuth token")


class InternalServerError(HTTPException):
    """Internal server error"""

    def __init__(self):
        super().__init__(500, "Internal server error")


class Unauthorized(HTTPException):
    """Unauthorized access"""

    def __init__(self):
        super().__init__(401, "Unauthorized access")
