from datetime import datetime
from typing import List

from fastapi import APIRouter, Depends, HTTPException, Query, status
from fastapi.security import OAuth2PasswordBearer
from pydantic import BaseModel, EmailStr, constr, validator
from sqlalchemy.exc import SQLAlchemyError

from ..auth import get_user, hash_password, verify_hashed_password
from ..database import SessionLocal, models

db = SessionLocal()
router = APIRouter(prefix="/users")


class User(BaseModel):
    id: int
    username: str
    fullname: str
    email: str


class UserCreate(BaseModel):
    username: str
    fullname: str
    email: EmailStr
    password: constr(min_length=8)

    @validator("password")
    def password_strength(cls, v):
        if not any(c.isupper() for c in v):
            raise ValueError("Password must contain an uppercase letter")
        if not any(c.islower() for c in v):
            raise ValueError("Password must contain a lowercase letter")
        if not any(c.isdigit() for c in v):
            raise ValueError("Password must contain a number")
        return v


@router.put("/", response_model=User, status_code=200)
async def update_user_details(
    new_entry: UserCreate,
    user=Depends(get_user),
):
    try:
        user_entry_to_update = (
            db.query(model.User).filter(models.User.id == user.id).first()
        )

        if user_entry_to_update is None:
            raise HTTPException(
                status_code=400, detail=f"User with the id {user.id} was not found"
            )

        hashed_password = await hash_password(new_entry.password)
        user_entry_to_update.username = new_entry.username
        user_entry_to_update.fullname = new_entry.fullname
        user_entry_to_update.email = new_entry.email
        user_entry_to_update.password = hashed_password
        db.commit()

        return User(
            id=user_entry_to_update.id,
            username=user_entry_to_update.username,
            fullname=user_entry_to_update.fullname,
            email=user_entry_to_update.email,
        )

    except SQLAlchemyError:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)


class DeletionSuccess(BaseModel):
    status: str = "Success"
    message: str = "User deleted successfully."


@router.delete("/", response_model=DeletionSuccess, status_code=200)
async def delete_user_detail(user=Depends(get_user)):
    try:
        user_entry_to_delete = (
            db.query(models.User).filter(models.User.id == user.id).first()
        )

        if user_entry_to_delete is None:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail=f"User with the id {user.id} was not found",
            )

        db.delete(user_entry_to_delete)
        db.commit()
        return DeletionSuccess()

    except SQLAlchemyError:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="User deletion was not successful",
        )
