use crate::components::{Echo, Hero};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    rsx! {
        Hero {}
        Echo {}
        div { 
            style: "text-align: center; margin: 2rem;",
            Link { 
                to: "/review", 
                class: "nav-link",
                style: "background: #007bff; color: white; padding: 1rem 2rem; text-decoration: none; border-radius: 5px; font-size: 1.1rem;",
                "Start Application Review" 
            }
        }
    }
}
