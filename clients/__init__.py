from ..config import settings
from .tripit import TripIt

CLIENTS = {}


def get_tripit_client() -> TripIt:
    if "tripit_client" not in CLIENTS:
        CLIENTS["tripit_client"] = TripIt(
            settings.CONSUMER_KEY, settings.CONSUMER_SECRET
        )
    return CLIENTS["tripit_client"]
