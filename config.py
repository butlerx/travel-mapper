from functools import lru_cache

from dotenv import load_dotenv
from pydantic_settings import BaseSettings, SettingsConfigDict

load_dotenv(".env")


class Settings(BaseSettings):
    app_name: str = "Travel Mapper"
    JWT_SECRET: str
    JWT_ALGORITHM: str
    CONSUMER_KEY: str
    CONSUMER_SECRET: str
    PORT: int = 8080

    class Config:
        env_file = ".env"


@lru_cache()
def get_settings():
    return Settings()


settings = get_settings()
