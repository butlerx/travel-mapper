from datetime import datetime

from sqlalchemy import Column, DateTime, ForeignKey, Integer, String

from .base import Base


class OauthState(Base):
    __tablename__ = "oauth_states"

    request_token = Column(String, primary_key=True)
    request_secret = Column(String)
    timestamp = Column(DateTime, default=datetime.now)
    user_id = Column(Integer, ForeignKey("users.id"))

    def __repr__(self):
        return (
            f"<OauthState(request_token={self.request_token}, user_id={self.user_id})>"
        )


class UserTokens(Base):
    __tablename__ = "user_tokens"

    access_token = Column(String, primary_key=True)
    access_secret = Column(String, nullable=False)
    user_id = Column(Integer, ForeignKey("users.id"))

    def __repr__(self):
        return f"<UserTokens(access_token={self.access_token}, user_id={self.user_id})>"
