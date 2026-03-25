from functools import lru_cache

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="ITERUM_", extra="ignore")

    app_name: str = "iterum"
    redis_url: str = "redis://localhost:6379/0"
    memory_ttl_seconds: int = 7 * 24 * 60 * 60
    kalshi_tool_name: str = "kalshi.get_market"


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    return Settings()

