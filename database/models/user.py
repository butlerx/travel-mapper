from datetime import datetime

from sqlalchemy import Column, DateTime, Integer, String

from .base import Base


class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    username = Column(String, unique=True, index=True)
    hashed_password = Column(String)
    access_token = Column(String, nullable=True)
    access_secret = Column(String, nullable=True)
    created_at = Column(DateTime, default=datetime.now)

    def __repr__(self):
        return f"<User(id={self.id}, username={self.username})>"
