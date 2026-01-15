use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Html},
    routing::{get, post},
    Form,
};

use nanoid::nanoid;
use serde::Deserialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {

    //Initialize the Tracing-subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = AppState {
        db: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(handler))
        .route("/shorten", post(shorten))
        .route("/{id}", get(redirect_url))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
 
    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    
    tracing::info!("Listening On {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

// New: Homepage with HTML form
async fn handler() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>URL Shortener</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 100px auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        .container {
            background: white;
            padding: 40px;
            border-radius: 10px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
        }
        h1 {
            color: #333;
            text-align: center;
            margin-bottom: 30px;
        }
        input[type="url"] {
            width: 100%;
            padding: 15px;
            font-size: 16px;
            border: 2px solid #ddd;
            border-radius: 5px;
            box-sizing: border-box;
            margin-bottom: 15px;
        }
        input[type="url"]:focus {
            outline: none;
            border-color: #667eea;
        }
        button {
            width: 100%;
            padding: 15px;
            font-size: 16px;
            background: #667eea;
            color: white;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            font-weight: bold;
        }
        button:hover {
            background: #5568d3;
        }
        #result {
            margin-top: 20px;
            padding: 15px;
            background: #f0f9ff;
            border-radius: 5px;
            display: none;
            word-break: break-all;
        }
        #result a {
            color: #667eea;
            text-decoration: none;
            font-weight: bold;
        }
        #result a:hover {
            text-decoration: underline;
        }
        .success {
            color: #10b981;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔗 URL Shortener</h1>
        <form id="shortenForm">
            <input 
                type="url" 
                name="url" 
                placeholder="Enter your long URL here (e.g., https://example.com)" 
                required
            />
            <button type="submit">Shorten URL</button>
        </form>
        <div id="result"></div>
    </div>

    <script>
        document.getElementById('shortenForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            
            const form = e.target;
            const url = form.url.value;
            const resultDiv = document.getElementById('result');
            
            try {
                const response = await fetch('/shorten', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded',
                    },
                    body: `url=${encodeURIComponent(url)}`
                });
                
                const shortUrl = await response.text();
                
                resultDiv.innerHTML = `
                    <p class="success">✓ Success! Your shortened URL:</p>
                    <p><a href="${shortUrl}" target="_blank">${shortUrl}</a></p>
                    <button onclick="copyToClipboard('${shortUrl}')" style="margin-top:10px; width:auto; padding:10px 20px;">
                        📋 Copy to Clipboard
                    </button>
                `;
                resultDiv.style.display = 'block';
                form.reset();
            } catch (error) {
                resultDiv.innerHTML = `<p style="color:red;">Error: ${error.message}</p>`;
                resultDiv.style.display = 'block';
            }
        });

        function copyToClipboard(text) {
            navigator.clipboard.writeText(text).then(() => {
                alert('Copied to clipboard!');
            });
        }
    </script>
</body>
</html>
    "#)
}

// Modified: Accept both JSON and Form data
async fn shorten(
    State(state): State<AppState>, 
    Form(payload): Form<CreateRequest>
) -> String {
    let id = nanoid!(6);

    let mut db = state.db.lock().unwrap();

    db.insert(id.clone(), payload.url);

    format!("http://localhost:7878/{}", id)
}

async fn redirect_url(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let db = state.db.lock().unwrap();

    if let Some(url) = db.get(&id) {
        Redirect::to(url).into_response()
    } else {
        (StatusCode::NOT_FOUND, "ID not found").into_response()
    }
}

#[derive(Deserialize)]
struct CreateRequest {
    url: String,
}

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<HashMap<String, String>>>,
}