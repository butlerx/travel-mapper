from datetime import datetime, timedelta
from typing import Tuple

from jose import jwt
from passlib.context import CryptContext

SECRET_KEY = "secret"
ALGORITHM = "HS256"
password_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


def verify_password(plain_password: str, hashed_password: str):
    return password_context.verify(plain_password, hashed_password)


def get_password_hash(password: str):
    return password_context.hash(password)


def hash_password(password: str):
    return password_context.hash(password)


def create_access_token(
    user_id: str, username: str, expires_delta: timedelta = timedelta(days=1)
) -> Tuple[str, str]:
    """Creates a new JWT token"""
    expires = datetime.utcnow() + expires_delta
    token_data = {
        "sub": str(user_id),
        "username": username,
        "exp": expires,
        "iat": datetime.utcnow(),
    }
    max_age = expires_delta.total_seconds()

    return jwt.encode(token_data, SECRET_KEY, algorithm=ALGORITHM), str(max_age)


def decode_access_token(token: str) -> dict:
    return jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
