from __future__ import annotations

from typing import NamedTuple

import uvicorn
from fastapi import Depends, FastAPI, Request
from fastapi.responses import HTMLResponse, ORJSONResponse, RedirectResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from .auth import get_user
from .config import settings
from .routes import login, oauth, register, trips, user
from .templates import templates

app = FastAPI()
app.mount("/static", StaticFiles(directory="static"), name="static")
app.include_router(user.router)
app.include_router(login.router)
app.include_router(oauth.router)
app.include_router(register.router)
app.include_router(trips.router)


@app.get("/", response_class=HTMLResponse)
async def home(request: Request, user=Depends(get_user)):
    return templates.TemplateResponse("home.html", {"request": request, "user": user})


class HealthResponse(BaseModel):
    status: str


@app.get("/health")
async def health_check() -> HealthResponse:
    return ORJSONResponse({"status": "ok 👍 "})


if __name__ == "__main__":
    port = int(settings.PORT)
    app_module = "main:app"
    uvicorn.run(app_module, host="0.0.0.0", port=port, reload=True)
