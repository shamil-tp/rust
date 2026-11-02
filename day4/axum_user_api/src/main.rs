use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client, Database,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::services::ServeDir;

// ---------------------------------------------------------
// 1. SCHEMAS (Equivalent to Mongoose Schema & Virtuals)
// ---------------------------------------------------------

// This matches the incoming POST request body
#[derive(Deserialize)]
struct CreateUserDto {
    username: String,
    email: String,
    password: String,
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
    bio: Option<String>,
}

// This is the actual MongoDB Document schema
#[derive(Debug, Serialize, Deserialize)]
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    username: String,
    email: String,
    password: String,
    #[serde(rename = "firstName", skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    #[serde(rename = "lastName", skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bio: Option<String>,
    role: String,
}

// This is what we send back to the client (stripping password, adding fullName)
#[derive(Serialize)]
struct UserResponse {
    username: String,
    email: String,
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
    #[serde(rename = "fullName")]
    full_name: String,
    bio: Option<String>,
    role: String,
}

// Logic to convert a DB User into a safe UserResponse (like Mongoose Virtuals + select: false)
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let first = user.first_name.clone().unwrap_or_default();
        let last = user.last_name.clone().unwrap_or_default();
        let full_name = if !first.is_empty() && !last.is_empty() {
            format!("{} {}", first, last)
        } else if !first.is_empty() {
            first
        } else if !last.is_empty() {
            last
        } else {
            user.username.clone()
        };

        Self {
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            full_name,
            bio: user.bio,
            role: user.role,
        }
    }
}

// ---------------------------------------------------------
// 2. QUERY PARAMS (For the GET endpoint)
// ---------------------------------------------------------
#[derive(Deserialize)]
struct AuthQuery {
    username: String,
    password: Option<String>, // Option used to handle missing param gracefully
}

// ---------------------------------------------------------
// 3. APPLICATION STATE (Holds the DB connection)
// ---------------------------------------------------------
#[derive(Clone)]
struct AppState {
    db: Database,
}

// ---------------------------------------------------------
// 4. API HANDLERS
// ---------------------------------------------------------

// POST /api/users
async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<User>("users");

    // Check if user already exists
    let filter = doc! {
        "$or": [
            { "username": &payload.username },
            { "email": &payload.email }
        ]
    };

    if let Ok(Some(_)) = collection.find_one(filter).await {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "A user with this username or email already exists" })),
        ));
    }

    // Hash password
    let hashed_password = bcrypt::hash(&payload.password, 10).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to hash password" })),
        )
    })?;

    // Create new user object
    let new_user = User {
        id: None,
        username: payload.username,
        email: payload.email,
        password: hashed_password,
        first_name: payload.first_name,
        last_name: payload.last_name,
        bio: payload.bio,
        role: "user".to_string(),
    };

    // Insert into DB
    collection.insert_one(&new_user).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Database insertion failed" })),
        )
    })?;

    // Return the safe response
    let response_data = UserResponse::from(new_user);
    Ok((StatusCode::CREATED, Json(response_data)))
}

// GET /api/user
async fn get_user(
    State(state): State<AppState>,
    Query(query): Query<AuthQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let password = query.password.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Username and password are required" })),
        )
    })?;

    let collection = state.db.collection::<User>("users");

    // Find user by username
    let filter = doc! { "username": query.username };
    let user = collection.find_one(filter).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Database error" })),
        )
    })?;

    if let Some(user) = user {
        // Verify bcrypt hash
        let is_valid = bcrypt::verify(&password, &user.password).unwrap_or(false);
        
        if is_valid {
            let response_data = UserResponse::from(user);
            return Ok(Json(response_data));
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Invalid credentials or user not found" })),
    ))
}

// ---------------------------------------------------------
// 5. SERVER SETUP & ROUTING
// ---------------------------------------------------------
#[tokio::main]
async fn main() {
    // Connect to MongoDB
    let mongo_uri = "mongodb://127.0.0.1:27017";
    let client = Client::with_uri_str(mongo_uri).await.expect("Failed to connect to MongoDB");
    let db = client.database("my_database"); // Matches the Express DB name

    let state = AppState { db };

    // Build the Axum router
    let app = Router::new()
        .route("/api/users", post(create_user))
        .route("/api/user", get(get_user))
        // Serve the static index.html from the "public" folder
        .fallback_service(ServeDir::new("public"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("Axum server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}