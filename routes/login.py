from typing import NamedTuple, Optional

from fastapi import APIRouter, Depends, Form, HTTPException, Request, Response
from fastapi.responses import HTMLResponse, RedirectResponse
from fastapi.security import OAuth2PasswordBearer
from pydantic import BaseModel

from ..auth import decode_jwt, sign_jwt, verify_hashed_password
from ..database import SessionLocal, models
from ..templates import templates

router = APIRouter(prefix="/login")
db = SessionLocal()
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/login")


@router.get("/", response_class=HTMLResponse)
async def login_page(
    request: Request,
    token: Optional[str] = Depends(oauth2_scheme),
) -> HTMLResponse:
    access_token = request.cookies.get("access_token")
    if access_token:
        user_from_token = decode_jwt(token)
        if user_from_token:
            return RedirectResponse(url="/")

    return templates.TemplateResponse("login.html", {"request": request, "error": None})


class Login(BaseModel):
    email: str
    password: str


class LoginInfo(NamedTuple):
    data: Login
    is_form: bool


async def get_login_data(
    request: Request,
    login: Optional[Login] = None,
    email: Optional[str] = Form(default=None),
    password: Optional[str] = Form(default=None),
):
    content_type = request.headers.get("content-type", "")
    if "form" in content_type:
        if email is None or password is None:
            raise HTTPException(
                status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
                detail="Email and password are required",
            )
        return LoginInfo(data=Login(email=email, password=password), is_form=True)

    if login is None:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Invalid request format",
        )
    return LoginInfo(data=login, is_form=False)


class TokenResponce(BaseModel):
    access_token: str


@router.post("/", response_model=TokenResponce)
async def login(request: Request, login_info: LoginInfo = Depends(get_login_data)):
    try:
        db_user = (
            db.query(models.User)
            .filter(models.User.email == login_info.data.email)
            .first()
        )

        if db_user is not None:
            is_password_valid = await verify_hashed_password(
                login_info.data.password, db_user.password
            )
            if is_password_valid:
                access_token = sign_jwt(db_user.id, db_user.email)

                if login_info.is_form:
                    response = RedirectResponse(url="/", status_code=303)
                    # Set the JWT as an HTTP-only cookie
                    response.set_cookie(
                        key="access_token",
                        value=access_token,
                        httponly=True,
                        max_age=1800,  # 30 minutes
                        secure=True,  # Use in production with HTTPS
                    )
                    return response
                return TokenResponce(access_token=access_token)
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid email or password",
        )
    except HTTPException as e:
        if login_info.is_form:
            return templates.TemplateResponse(
                "login.html",
                {"request": request, "error": e.detail},
                status_code=e.status_code,
            )
        raise e
    except SQLAlchemyError:
        if login_info.is_form:
            return templates.TemplateResponse(
                "login.html",
                {"request": request, "error": "Invalid email or password"},
                status_code=status.HTTP_401_UNAUTHORIZED,
            )
        raise HTTPException(
            status_code=status.http_401_unauthorized,
            detail="Invalid email or password",
        )
