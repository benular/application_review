use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use views::{Blog, Home, Navbar};

#[cfg(feature = "server")]
use mongodb::{Client, Collection};
#[cfg(feature = "server")]
use std::env;

/// Define a components module that contains all shared components for our app.
mod components;
/// Define a views module that contains the UI for all Layouts and Routes for our app.
mod views;

/// The Route enum is used to define the structure of internal routes in our app. All route enums need to derive
/// the [`Routable`] trait, which provides the necessary methods for the router to work.
/// 
/// Each variant represents a different URL pattern that can be matched by the router. If that pattern is matched,
/// the components for that route will be rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    // The layout attribute defines a wrapper for all routes under the layout. Layouts are great for wrapping
    // many routes with a common UI like a navbar.
    #[layout(Navbar)]
        // The route attribute defines the URL pattern that a specific route matches. If that pattern matches the URL,
        // the component for that route will be rendered. The component name that is rendered defaults to the variant name.
        #[route("/")]
        Home {},
        // The route attribute can include dynamic parameters that implement [`std::str::FromStr`] and [`std::fmt::Display`] with the `:` syntax.
        // In this case, id will match any integer like `/blog/123` or `/blog/-456`.
        #[route("/blog/:id")]
        Blog { id: i32 },
        #[route("/review")]
        ReviewPage {},
}

// We can import assets in dioxus with the `asset!` macro. This macro takes a path to an asset relative to the crate root.
// The macro returns an `Asset` type that will display as the path to the asset in the browser or a local path in desktop bundles.
const FAVICON: Asset = asset!("/assets/favicon.ico");
// The asset macro also minifies some assets like CSS and JS to make bundled smaller
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const REVIEW_CSS: Asset = asset!("/assets/styling/review.css");

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    dioxus::launch(App);
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
fn App() -> Element {
    // The `rsx!` macro lets us define HTML inside of rust. It expands to an Element with all of our HTML inside.
    rsx! {
        // In addition to element and text (which we will see later), rsx can contain other components. In this case,
        // we are using the `document::Link` component to add a link to our favicon and main CSS file into the head of our app.
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: REVIEW_CSS }


        // The router component renders the route enum we defined above. It will handle synchronization of the URL and render
        // the layouts and components for the active route.
        Router::<Route> {}
    }
}
#[component]
fn ReviewPage() -> Element {
    let mut reviews = use_signal(|| Vec::<Review>::new());
    let mut loading = use_signal(|| true);
    let mut submission_status = use_signal(|| String::new());

    use_effect(move || {
        spawn(async move {
            match load_questions().await {
                Ok(loaded_reviews) => {
                    reviews.set(loaded_reviews);
                    loading.set(false);
                }
                Err(e) => {
                    submission_status.set(format!("Error loading questions: {}", e));
                    loading.set(false);
                }
            }
        });
    });

    let submit_reviews = move |_| {
        let reviews_data = reviews.read().clone();
        spawn(async move {
            #[cfg(feature = "server")]
            {
                match submit_to_mongodb(reviews_data).await {
                    Ok(_) => submission_status.set("Reviews submitted successfully!".to_string()),
                    Err(e) => submission_status.set(format!("Error submitting reviews: {}", e)),
                }
            }
            #[cfg(not(feature = "server"))]
            {
                // For web-only builds, just show the data
                submission_status.set(format!("Would submit {} reviews (web build)", reviews_data.len()));
            }
        });
    };

    if loading() {
        return rsx! {
            div { class: "loading", "Loading questions..." }
        };
    }

    rsx! {
        div { class: "review-container",
            h1 { "Application Review" }
            
            if !submission_status().is_empty() {
                div { class: "status-message", "{submission_status()}" }
            }
            
            form {
                onsubmit: submit_reviews,
                
                for (index, review) in reviews().iter().enumerate() {
                    div { class: "question-block", key: "{index}",
                        if !review.question.trim().is_empty() {
                            div {
                                h3 { "Category: {review.category}" }
                                p { class: "question", "{review.question}" }
                                
                                StarRating {
                                    initial_rating: review.rating as f32,
                                    on_rate: move |rating| {
                                        let mut current_reviews = reviews.write();
                                        if let Some(review_mut) = current_reviews.get_mut(index) {
                                            review_mut.rating = rating as u8;
                                        }
                                    }
                                }
                                
                                div { class: "advice-section",
                                    label { "Your advice:" }
                                    textarea {
                                        value: "{review.advice}",
                                        placeholder: "Enter your advice here...",
                                        oninput: move |event| {
                                            let mut current_reviews = reviews.write();
                                            if let Some(review_mut) = current_reviews.get_mut(index) {
                                                review_mut.advice = event.value();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                button { 
                    r#type: "submit",
                    class: "submit-btn",
                    "Submit Reviews"
                }
            }
        }
    }
}
#[component]
pub fn StarRating(initial_rating: Option<f32>, on_rate: Option<EventHandler<f32>>,) -> Element {
    let mut rating = use_signal(|| initial_rating.unwrap_or(0.0));
    let mut hover_rating = use_signal(|| 0.0f32);

    rsx! {
        div {
            class: "star-rating",
            style: "display: inline-flex; gap: 4px; cursor: pointer; user-select: none;",
            
            for star_index in 1..=5 {
                span {
                    class: "star",
                    style: "font-size: 2rem; transition: color 0.2s ease; position: relative;",
                    onmouseenter: move |_| {
                        hover_rating.set(star_index as f32);
                    },
                    onmouseleave: move |_| hover_rating.set(0.0),
                    onclick: move |_| {
                        let new_rating = star_index as f32;
                        rating.set(new_rating);
                        if let Some(handler) = &on_rate {
                            handler.call(new_rating);
                        }
                    },
                    
                    {render_star(star_index as f32, hover_rating(), rating())}
                }
            }
            
            span {
                style: "margin-left: 10px; color: #666;",
                "{rating():.1}/5.0"
            }
        }
    }
}

fn render_star(star_index: f32, hover: f32, rating: f32) -> &'static str {
    let current_rating = if hover > 0.0 { hover } else { rating };
    
    if current_rating >= star_index {
        "★"
    } else if current_rating >= star_index - 0.5 {
        "⯨" 
    } else {
        "☆"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Review {
    pub category: String,
    pub question: String,
    pub rating: u8,
    pub advice: String,
}

#[derive(Debug, Deserialize)]
struct QuestionsData {
    reviews: Vec<Review>,
}

async fn load_questions() -> std::result::Result<Vec<Review>, Box<dyn std::error::Error>> {
    let questions_json = include_str!("../assets/questions.json");
    let data: QuestionsData = serde_json::from_str(questions_json)?;
    Ok(data.reviews)
}

#[cfg(feature = "server")]
async fn submit_to_mongodb(reviews: Vec<Review>) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mongodb_uri = env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://appuser:apppassword@localhost:27017/applications?authSource=applications".to_string());
    
    let client = Client::with_uri_str(&mongodb_uri).await?;
    let database = client.database("applications");
    let collection: Collection<Review> = database.collection("reviews");
    
    collection.insert_many(reviews, None).await?;
    Ok(())
}
