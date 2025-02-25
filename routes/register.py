from typing import NamedTuple, Optional

from fastapi import APIRouter, Depends, Form, HTTPException, Request, Response, status
from fastapi.responses import HTMLResponse, RedirectResponse
from fastapi.security import OAuth2PasswordBearer
from pydantic import BaseModel, EmailStr, constr, validator

from ..auth import decode_jwt, sign_jwt, verify_hashed_password
from ..database import SessionLocal, models
from ..templates import templates
from .user import User, UserCreate

router = APIRouter(prefix="/register")
db = SessionLocal()
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/login")


@router.get("/", response_class=HTMLResponse)
async def register_page(
    request: Request,
    token: Optional[str] = Depends(oauth2_scheme),
) -> HTMLResponse:
    access_token = request.cookies.get("access_token")
    if access_token:
        user_from_token = decode_jwt(token)
        if user_from_token:
            return RedirectResponse(url="/")
    return templates.TemplateResponse(
        "register.html", {"request": request, "error": None}
    )


class RegisterInfo(NamedTuple):
    data: UserCreate
    is_form: bool


async def get_register_data(
    request: Request,
    user: Optional[UserCreate] = None,
    username: Optional[str] = Form(...),
    fullname: Optional[str] = Form(...),
    email: Optional[str] = Form(...),
    password: Optional[str] = Form(...),
):
    content_type = request.headers.get("content-type", "")
    if "form" in content_type:
        return RegisterInfo(
            data=UserCreate(
                username=username,
                fullname=fullname,
                email=email,
                password=password,
            ),
            is_form=True,
        )

    if login is None:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Invalid request format",
        )
    return RegisterInfo(data=user, is_form=False)


@router.post("/", response_model=User, status_code=status.HTTP_201_CREATED)
async def register(
    request: Request, register_info: RegisterInfo = Depends(get_register_data)
):
    try:
        user = register_info.data
        hashed_password = await hash_password(user.password)

        new_user = models.User(
            username=user.username,
            fullname=user.fullname,
            email=user.email,
            password=hashed_password,
        )

        # Check if a user with the same email already exists
        db_item = (
            db.query(models.User).filter(models.User.email == new_user.email).first()
        )

        if db_item is not None:
            raise HTTPException(
                status_code=400, detail="User with the email already exists"
            )

        # Add the new user to the database
        db.add(new_user)
        db.commit()

        if register_info.is_form:
            access_token = sign_jwt(new_user.id, new_user.email)
            response = RedirectResponse(url="/", status_code=303)
            response.set_cookie(
                key="access_token",
                value=access_token,
                httponly=True,
                max_age=1800,  # 30 minutes
                secure=True,  # Use in production with HTTPS
            )
            return response
        return User(
            id=new_user.id,
            username=new_user.username,
            fullname=new_user.fullname,
            email=new_user.email,
        )

    except HTTPException as e:
        if register_info.is_form:
            return templates.TemplateResponse(
                "register.html",
                {"request": request, "error": e.detail},
                status_code=e.status_code,
            )
        raise e
