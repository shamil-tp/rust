from datetime import datetime
from typing import Optional
from fastapi import FastAPI, HTTPException, Query, status
from fastapi.staticfiles import StaticFiles
from motor.motor_asyncio import AsyncIOMotorClient
from passlib.context import CryptContext
from pydantic import BaseModel, EmailStr, Field

# Initialize FastAPI app
app = FastAPI(title="User Management API")

# Password hashing context (Bcrypt)
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

# MongoDB Connection Setup
MONGO_URL = "please paste uri"
client = AsyncIOMotorClient(MONGO_URL)
db = client.grafanka  # Matches your Express & Axum DB name
users_collection = db.users

# ---------------------------------------------------------
# 1. SCHEMAS (Pydantic Models matching Mongoose)
# ---------------------------------------------------------

class UserCreate(BaseModel):
    username: str = Field(..., min_length=3, max_length=30)
    email: EmailStr
    password: str = Field(..., min_length=6)
    firstName: Optional[str] = Field(None, max_length=50)
    lastName: Optional[str] = Field(None, max_length=50)
    bio: Optional[str] = Field(None, max_length=500)

class UserResponse(BaseModel):
    username: str
    email: str
    firstName: Optional[str] = None
    lastName: Optional[str] = None
    fullName: str
    bio: Optional[str] = None
    role: str
    createdAt: Optional[datetime] = None

# Helper to convert MongoDB document to safe UserResponse (Virtuals + Password Stripping)
def user_helper(user) -> dict:
    first = user.get("firstName") or ""
    last = user.get("lastName") or ""
    
    if first and last:
        full_name = f"{first} {last}"
    elif first:
        full_name = first
    elif last:
        full_name = last
    else:
        full_name = user["username"]

    # Extract creation time from MongoDB ObjectId if createdAt isn't explicitly set
    created_at = user.get("createdAt")
    if not created_at and "_id" in user:
        created_at = user["_id"].generation_time

    return {
        "username": user["username"],
        "email": user["email"],
        "firstName": user.get("firstName"),
        "lastName": user.get("lastName"),
        "fullName": full_name,
        "bio": user.get("bio"),
        "role": user.get("role", "user"),
        "createdAt": created_at
    }

# ---------------------------------------------------------
# 2. API ENDPOINTS
# ---------------------------------------------------------

@app.post("/api/users", status_code=status.HTTP_201_CREATED)
async def create_user(payload: UserCreate):
    # Check if username or email already exists
    existing_user = await users_collection.find_one({
        "$or": [{"username": payload.username}, {"email": payload.email}]
    })
    
    if existing_user:
        raise HTTPException(
            status_code=400, 
            detail="A user with this username or email already exists"
        )

    # Hash the password
    hashed_password = pwd_context.hash(payload.password)

    # Prepare user document
    user_dict = payload.dict()
    user_dict["password"] = hashed_password
    user_dict["role"] = "user"
    user_dict["createdAt"] = datetime.utcnow()

    # Insert into MongoDB
    result = await users_collection.insert_one(user_dict)
    
    # Fetch the newly created document to return
    created_user = await users_collection.find_one({"_id": result.inserted_id})
    return {"message": "User created successfully", "user": user_helper(created_user)}


@app.get("/api/user")
async def get_user(
    username: str = Query(...), 
    password: str = Query(...)
):
    # Find user by username
    user = await users_collection.find_one({"username": username})
    
    if not user:
        raise HTTPException(
            status_code=401, 
            detail="Invalid credentials or user not found"
        )

    # Verify password hash
    if not pwd_context.verify(password, user["password"]):
        raise HTTPException(
            status_code=401, 
            detail="Invalid credentials or user not found"
        )

    return user_helper(user)

# ---------------------------------------------------------
# 3. STATIC FILE SERVING (Serves index.html from 'public')
# ---------------------------------------------------------
app.mount("/", StaticFiles(directory="public", html=True), name="public")