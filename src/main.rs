use dioxus::prelude::*;
use route_transitions::MotionTransitions;

pub mod will_hide;
pub use will_hide::*;

#[derive(Routable, Clone, Debug, PartialEq, MotionTransitions)]
#[rustfmt::skip]
enum Route {
    #[layout(AnimationBuilder)]
        #[route("/")]
        #[transition(Fade)]
        Home {},
        #[route("/slide-left")]
        #[transition(ZoomIn)]
        SlideLeft {},
        #[route("/slide-right")]
        #[transition(SlideRight)]
        SlideRight {},
        #[route("/slide-up")]
        #[transition(SlideUp)]
        SlideUp {},
        #[route("/slide-down")]
        #[transition(SlideDown)]
        SlideDown {},
        #[route("/fade")]
        #[transition(Fade)]
        Fade {},
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
pub fn AnimationBuilder() -> Element {
    rsx! {
        AnimatedRouter::<Route> {}
    }
}

#[component]
fn NavLink(to: Route, children: Element) -> Element {
    let current_route = use_route::<Route>();
    let is_active = current_route == to;

    rsx! {
        Link {
            to,
            class: if is_active { "px-3 py-1 rounded-full text-sm transition-colors duration-200 bg-indigo-100 text-indigo-700" } else { "px-3 py-1 rounded-full text-sm transition-colors duration-200 text-gray-600 hover:text-gray-900 hover:bg-gray-100" },
            {children}
        }
    }
}

#[component]
fn TransitionPage(
    title: &'static str,
    description: &'static str,
    bg_color: &'static str,
    accent: &'static str,
    children: Element,
) -> Element {
    rsx! {
        div { class: "min-h-screen pt-16 {bg_color}",
            div { class: "max-w-4xl mx-auto p-8",
                div { class: "bg-white/80 backdrop-blur-md rounded-2xl shadow-xl p-8 border border-gray-100",
                    h1 { class: "text-4xl font-bold mb-4 {accent}", "{title}" }
                    p { class: "text-gray-600 text-lg", "{description}" }
                    div { class: "mt-8", {children} }
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        TransitionPage {
            title: "Welcome to Page Transitions in Dioxus",
            description: "Start exploring different transition animations",
            bg_color: "bg-gradient-to-br from-blue-50 to-indigo-100",
            accent: "text-blue-600",
            NavLink { to: Route::SlideLeft {}, "Start with Slide Left →" }
        }
    }
}

#[component]
fn SlideLeft() -> Element {
    rsx! {
        TransitionPage {
            title: "Slide Left Transition",
            description: "Next, let's try sliding right",
            bg_color: "bg-gradient-to-br from-red-50 to-pink-100",
            accent: "text-red-600",
            NavLink { to: Route::SlideRight {}, "Try Slide Right →" }
        }
    }
}

#[component]
fn SlideRight() -> Element {
    rsx! {
        TransitionPage {
            title: "Slide Right Transition",
            description: "Now, let's slide upwards",
            bg_color: "bg-gradient-to-br from-green-50 to-emerald-100",
            accent: "text-green-600",
            NavLink { to: Route::SlideUp {}, "Try Slide Up ↑" }
        }
    }
}

#[component]
fn SlideUp() -> Element {
    rsx! {
        TransitionPage {
            title: "Slide Up Transition",
            description: "Let's try sliding down",
            bg_color: "bg-gradient-to-br from-purple-50 to-violet-100",
            accent: "text-purple-600",
            NavLink { to: Route::SlideDown {}, "Try Slide Down ↓" }
        }
    }
}

#[component]
fn SlideDown() -> Element {
    rsx! {
        TransitionPage {
            title: "Slide Down Transition",
            description: "Finally, let's try fading",
            bg_color: "bg-gradient-to-br from-yellow-50 to-amber-100",
            accent: "text-yellow-600",
            NavLink { to: Route::Fade {}, "Try Fade Effect" }
        }
    }
}

#[component]
fn Fade() -> Element {
    rsx! {
        TransitionPage {
            title: "Fade Transition",
            description: "That's all the transitions! Start over?",
            bg_color: "bg-gradient-to-br from-orange-50 to-rose-100",
            accent: "text-orange-600",
            div { class: "space-y-4",
                NavLink { to: Route::Home {}, "← Back to Start" }
            }
        }
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattemped to navigate to: {route:?}" }
    }
}

const MAIN_CSS: Asset = asset!("/assets/main.css");
fn main() {
    dioxus::launch(|| {
        rsx! {
            document::Link { rel: "stylesheet", href: MAIN_CSS }
            Router::<Route> {}
        }
    });
}
