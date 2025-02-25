from datetime import datetime, timedelta
from typing import List, Optional

from fastapi import APIRouter, Depends, HTTPException, Query, Request, status
from fastapi.responses import ORJSONResponse, RedirectResponse
from pydantic import BaseModel
from sqlalchemy.exc import SQLAlchemyError
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import Session, relationship, sessionmaker

from ..auth import get_user
from ..clients import get_tripit_client
from ..config import settings
from ..database import SessionLocal, models

# Create a database session
db = SessionLocal()
router = APIRouter(prefix="/oauth")


class InitiateResponse(BaseModel):
    redirect_to: str


@router.post("/initiate")
async def initiate_oauth(
    tripit_client: str = Depends(get_tripit_client), user=Depends(get_user)
) -> InitiateResponse:
    request_token, request_secret = await tripit_client.get_request_token()
    oauth_state = models.OauthState(
        request_token=request_token, request_secret=request_secret, user_id=user.id
    )
    db.add(oauth_state)
    auth_url = tripit_client.get_authorization_url(
        request_token, f"http://{settings.DOMAIN_NAME}/oauth/callback"
    )
    db.commit()
    return InitiateResponse(redirect_to=auth_url)


@router.get("/callback")
async def oauth_callback(
    oauth_token: str, tripit_client: str = Depends(get_tripit_client)
):
    """Handle OAuth callback from TripIt"""
    state = db.query(OauthState).filter(OauthState.request_token == oauth_token).first()
    if not state:
        raise HTTPException(status_code=401, detail="Invalid OAuth token")
    if datetime.now() - state.timestamp > timedelta(minutes=30):
        db.delete(state)
        db.commit()
        raise HTTPException(
            status_code=400, detail="OAuth token has expired. Please try again."
        )

    try:
        access_token, access_secret = await tripit_client.get_access_token(
            state.request_token, state.request_secret
        )
    except Exception as e:
        logger.error("Failed to get Access token: %s" % str(e))
        raise HTTPException(status_code=401, detail="Invalid OAuth token")

    user_token = UserTokens(
        access_token=access_token, access_secret=access_secret, user_id=state.user_id
    )
    db.add(user_token)
    db.delete(state)
    db.commit()
    return RedirectResponse(url="/oauth/success")
