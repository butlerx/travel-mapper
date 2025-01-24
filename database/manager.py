from contextlib import contextmanager
from typing import Dict, Optional, Tuple

from sqlalchemy.orm import Session

from .models import SessionLocal, User
from .security import create_access_token, hash_password, verify_password


class DatabaseManager:
    session: Session

    def __init__(self, session: Session):
        self.session = session

    def get_user(self, user_id: int) -> User:
        return self.session.query(User).filter(User.id == user_id).first()

    def get_user_by_username(self, username: str) -> User:
        return self.session.query(User).filter(User.username == username).first()

    def create_user(self, user: Dict) -> User:
        user = User(
            username=user["username"],
            hashed_password=hash_password(user["password"]),
        )
        self.session.add(user)
        self.session.commit()
        self.session.refresh(user)
        return user

    def authenticate_user(
        self, username: str, password: str
    ) -> Optional[Tuple[str, str]]:
        user = self.get_user_by_username(username)
        if user is None or not verify_password(password, user.hashed_password):
            return None

        return create_access_token(user.id, user.username)


@contextmanager
def database_manager():
    with SessionLocal() as session:
        yield DatabaseManager(session)
