from robyn import Robyn

from .oauth import oauth_router
from .trips import trips_router
from .users import users_router


def setup_routes(app: Robyn) -> None:
    app.include_router(oauth_router)
    app.include_router(trips_router)
    app.include_router(users_router)
