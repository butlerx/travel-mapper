import time
from typing import Dict, NamedTuple, Optional

import argon2.exceptions
import jwt
from argon2 import PasswordHasher
from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer

from .config import settings

JWT_SECRET = settings.JWT_SECRET
JWT_ALGORITHM = settings.JWT_ALGORITHM
expiration_time = time.time() + (16 * 60 * 60)  # 16 hours (60 minutes * 60 seconds)
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/login")
ph = PasswordHasher()


async def hash_password(password: str) -> str:
    hashed_password = ph.hash(password)
    return hashed_password


async def verify_hashed_password(password: str, hashed_password: str) -> bool:
    try:
        return ph.verify(hashed_password, password)
    except argon2.exceptions.VerifyMismatchError:
        return False


def sign_jwt(user_id: str, user_email: str) -> str:
    payload = {
        "user_id": user_id,
        "user_email": user_email,
        "expires": expiration_time,
    }

    return jwt.encode(payload, JWT_SECRET, JWT_ALGORITHM)


class UserFromToken(NamedTuple):
    id: str
    email: str


def decode_jwt(token: str) -> Optional[UserFromToken]:
    try:
        decoded_token = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])
        if decoded_token["expires"] >= time.time():
            return UserFromToken(
                id=decoded_token["user_id"], email=decoded_token["user_email"]
            )

        return None
    except:
        return None


def get_user(
    access_token: str = Depends(oauth2_scheme),
) -> UserFromToken:
    user_from_token = decode_jwt(token)
    if not user_from_token:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid or expired token"
        )
    return user_from_token
