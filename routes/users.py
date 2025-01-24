from time import time

from robyn import Request, Response, SubRouter
from robyn.logger import Logger
from robyn.robyn import QueryParams

from database import database_manager
from errors import Unauthorized
from templates import templates

logger = Logger()
users_router: SubRouter = SubRouter(__name__, prefix="/user")


@users_router.get("/register")
async def show_register(request: Request):
    return templates.render_template(
        "register.html", error=request.query_params.get("error")
    )


@users_router.post("/register")
async def register_user(request: Request):
    try:
        form_data = request.form_data
        user = {
            "username": form_data.get("username"),
            "password": form_data.get("password"),
        }

        with database_manager() as db:
            created_user = db.create_user(user)

        return Response(
            status_code=302,
            description="Registration successful",
            headers={"Location": "/user/login?message=Registration successful"},
        )
    except Exception as e:
        return templates.render_template("register.html", error=str(e))


@users_router.get("/login")
async def show_login(request: Request):
    # TODO check if user is already logged in
    if request.identity:
        return Response(
            status_code=302,
            description="Already logged in",
            headers={"Location": "/dashboard"},
        )
    return templates.render_template(
        "login.html",
        error=request.query_params.get("error"),
        message=request.query_params.get("message"),
    )


@users_router.post("/login")
async def login_user(request: Request):
    try:
        form_data = request.form_data
        user = {
            "username": form_data.get("username"),
            "password": form_data.get("password"),
        }

        with database_manager() as db:
            token = db.authenticate_user(**user)

        if token is None:
            raise Unauthorized()

        token, max_age = token

        cookie_value = (
            f"auth_token={token}; "
            "Path=/; "
            "HttpOnly; "
            "Secure; "
            "SameSite=Lax; "
            f"Max-Age={max_age}"
        )
        return Response(
            status_code=302,
            description="Redirecting to dashboard",
            headers={
                "Location": "/dashboard",
                "Set-Cookie": cookie_value,
            },
        )
    except Unauthorized:
        return Response(
            status_code=302,
            description="Invalid credentials",
            headers={"Location": "/user/login?error=Invalid credentials"},
        )


@users_router.get("/logout")
async def logout(request: Request):
    return Response(
        status_code=302,
        description="Logged out successfully",
        headers={
            "Location": "/user/login?message=Logged out successfully",
            "Set-Cookie": "auth_token=; Max-Age=0",
        },
    )
