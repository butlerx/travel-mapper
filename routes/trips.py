import logging
from typing import Any, Dict, List

from fastapi import APIRouter, Depends, HTTPException, Request
from fastapi.security import OAuth2PasswordBearer
from sqlalchemy.orm import Session

from ..auth import get_user
from ..clients import get_tripit_client
from ..database import get_db, models

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api/trips")


@router.get("/")
async def get_trips(
    db: Session = Depends(get_db),
    tripit_client=Depends(get_tripit_client),
    user=Depends(get_user),
):
    """Get trips from TripIt API"""
    if not access_token:
        raise HTTPException(
            status_code=400, detail="Access token not setup please auth with tripit"
        )

    try:
        # Query the database for tokens using the access_token
        user_token = (
            db.query(models.UserTokens)
            .filter(models.UserTokens.user_id == user.id)
            .first()
        )

        if not user_token:
            raise HTTPException(status_code=401, detail="Invalid OAuth token")

        response = await tripit_client.list_trip(
            user_token.access_token, user_token.access_secret
        )
        return response

    except Exception as e:
        logger.error("Failed to get trips: %s", e)
        raise HTTPException(status_code=500, detail="An internal server error occurred")
