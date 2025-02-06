use dioxus::prelude::*;
use dioxus_motion::prelude::*;
use dioxus_router::prelude::{use_route, Routable};
use std::time::Duration;

#[derive(Clone)]
pub enum AnimatedRouterContext<R: Routable + PartialEq> {
    /// Transition from one route to another.
    FromTo(R, R),
    /// Settled in a route.
    In(R),
}

impl<R: Routable + PartialEq> AnimatedRouterContext<R> {
    /// Get the current destination route.
    pub fn target_route(&self) -> &R {
        match self {
            Self::FromTo(_, to) => to,
            Self::In(to) => to,
        }
    }

    /// Update the destination route.
    pub fn set_target_route(&mut self, to: R) {
        match self {
            Self::FromTo(old_from, old_to) => {
                *old_from = old_to.clone();
                *old_to = to
            }
            Self::In(old_to) => *self = Self::FromTo(old_to.clone(), to),
        }
    }

    /// After the transition animation has finished, make the outlet only render the destination route.
    pub fn settle(&mut self) {
        if let Self::FromTo(_, to) = self {
            *self = Self::In(to.clone())
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AnimatedRouterProps {
    children: Element,
}

/// Provide a mechanism for outlets to animate between route transitions.
///
/// See the `animated_sidebar.rs` or `animated_tabs.rs` for an example on how to use it.
#[allow(non_snake_case)]
pub fn AnimatedRouter<R: Routable + PartialEq + Clone>(
    AnimatedRouterProps { children }: AnimatedRouterProps,
) -> Element {
    let route = use_route::<R>();
    let mut prev_route = use_signal(|| AnimatedRouterContext::In(route.clone()));
    use_context_provider(move || prev_route);

    if prev_route.peek().target_route() != &route {
        prev_route.write().set_target_route(route);
    }

    rsx!(
        {children}
    )
}

/// Shortcut to get access to the [AnimatedRouterContext].
pub fn use_animated_router<Route: Routable + PartialEq>() -> Signal<AnimatedRouterContext<Route>> {
    use_context()
}

#[derive(PartialEq, Clone)]
pub enum TransitionVariant {
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Fade,
    Scale,
    Custom(Transform),
}

impl TransitionVariant {
    fn get_transform(&self) -> Transform {
        match self {
            TransitionVariant::SlideLeft => Transform::new(-100.0, 0.0, 1.0, 0.0),
            TransitionVariant::SlideRight => Transform::new(100.0, 0.0, 1.0, 0.0),
            TransitionVariant::SlideUp => Transform::new(0.0, -100.0, 1.0, 0.0),
            TransitionVariant::SlideDown => Transform::new(0.0, 100.0, 1.0, 0.0),
            TransitionVariant::Fade => Transform::new(0.0, 0.0, 0.0, 0.0),
            TransitionVariant::Scale => Transform::new(0.0, 0.0, 0.5, 0.0),
            TransitionVariant::Custom(transform) => *transform,
        }
    }
}
#[component]
fn FromRouteToCurrent(from: Element, transition: TransitionVariant) -> Element {
    let mut animated_router = use_animated_router::<Route>();
    let mut transform = use_motion(Transform::new(0.0, 0.0, 1.0, 1.0));
    let mut opacity = use_motion(1.0f32);

    // Initial transform setup
    use_effect(move || {
        // First set initial position
        transform.reset();
        opacity.reset();

        // Then animate to final position
        let target_transform = match transition {
            TransitionVariant::SlideLeft => Transform::new(-100.0, 0.0, 1.0, 0.0),
            TransitionVariant::SlideRight => Transform::new(100.0, 0.0, 1.0, 0.0),
            TransitionVariant::SlideUp => Transform::new(0.0, -100.0, 1.0, 0.0),
            TransitionVariant::SlideDown => Transform::new(0.0, 100.0, 1.0, 0.0),
            _ => transition.get_transform(),
        };

        let config = AnimationConfig::new(AnimationMode::Tween(
            Tween::new(Duration::from_millis(500)).with_easing(|t, b, c, d| {
                // Cubic ease-in-out
                let t = t / (d / 2.0);
                if t < 1.0 {
                    c / 2.0 * t * t * t + b
                } else {
                    let t = t - 2.0;
                    c / 2.0 * (t * t * t + 2.0) + b
                }
            }),
        ));

        transform.animate_to(target_transform, config.clone());
        opacity.animate_to(0.0, config);
    });

    use_effect(move || {
        if !transform.is_running() && !opacity.is_running() {
            animated_router.write().settle();
        }
    });

    rsx! {
        div {
            class: "",
            style: "
                   transform: translate({transform.get_value().x}%, {transform.get_value().y}%) 
                   scale({transform.get_value().scale});
                   opacity: {opacity.get_value()};
                   ",
            {from}
            Outlet::<Route> {}
        }
    }
}

#[component]
fn AnimatedOutlet(children: Element) -> Element {
    let animated_router = use_context::<Signal<AnimatedRouterContext<Route>>>();

    let from_route = match animated_router() {
        AnimatedRouterContext::FromTo(Route::Home {}, Route::SlideLeft {}) => {
            Some((rsx!(
                Home {}
            ), TransitionVariant::SlideLeft))
        }
        AnimatedRouterContext::FromTo(Route::SlideLeft {}, Route::Home {}) => {
            Some((rsx!(
                SlideLeft {}
            ), TransitionVariant::SlideRight))
        }
        AnimatedRouterContext::FromTo(Route::Home {}, Route::SlideRight {}) => {
            Some((rsx!(
                Home {}
            ), TransitionVariant::SlideRight))
        }
        AnimatedRouterContext::FromTo(Route::SlideRight {}, Route::Home {}) => {
            Some((rsx!(
                SlideRight {}
            ), TransitionVariant::SlideLeft))
        }
        AnimatedRouterContext::FromTo(Route::Home {}, Route::SlideUp {}) => {
            Some((rsx!(
                Home {}
            ), TransitionVariant::SlideUp))
        }
        AnimatedRouterContext::FromTo(Route::SlideUp {}, Route::Home {}) => {
            Some((rsx!(
                SlideUp {}
            ), TransitionVariant::SlideDown))
        }
        AnimatedRouterContext::FromTo(Route::Home {}, Route::SlideDown {}) => {
            Some((rsx!(
                Home {}
            ), TransitionVariant::SlideDown))
        }
        AnimatedRouterContext::FromTo(Route::SlideDown {}, Route::Home {}) => {
            Some((rsx!(
                SlideDown {}
            ), TransitionVariant::SlideUp))
        }
        AnimatedRouterContext::FromTo(Route::Home {}, Route::Fade {}) => {
            Some((rsx!(
                Home {}
            ), TransitionVariant::Fade))
        }
        AnimatedRouterContext::FromTo(Route::Fade {}, Route::Home {}) => {
            Some((rsx!(
                Fade {}
            ), TransitionVariant::Fade))
        }
        AnimatedRouterContext::FromTo(Route::Home {}, Route::Scale {}) => {
            Some((rsx!(
                Home {}
            ), TransitionVariant::Scale))
        }
        AnimatedRouterContext::FromTo(Route::Scale {}, Route::Home {}) => {
            Some((rsx!(
                Scale {}
            ), TransitionVariant::Scale))
        }
        _ => None,
    };

    rsx! {
        div {
            if let Some((from, transition)) = from_route {
                FromRouteToCurrent { from, transition }
            } else {
                Outlet::<Route> {}
            }
        }
    }
}

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(NavBar)]
        #[route("/")]
       
        Home {},
        #[route("/slide-left")]
        SlideLeft {},
        #[route("/slide-right")]
        SlideRight {},
        #[route("/slide-up")]
        SlideUp {},
        #[route("/slide-down")]
        SlideDown {},
        #[route("/fade")]
        Fade {},
        #[route("/scale")]
        Scale {},
    
    #[end_layout]

    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
fn NavBar() -> Element {
    rsx! {
        AnimatedRouter::<Route> {
            nav { class: "fixed top-0 left-0 right-0 z-50 bg-white/80 backdrop-blur-md border-b border-gray-200 px-4 py-3",
                div { class: "max-w-7xl mx-auto flex items-center justify-between",
                    span { class: "text-lg font-semibold text-gray-800",
                        "Dioxus Motion Page Transitions"
                    }
                    div { class: "flex gap-4 items-center",
                        NavLink { to: Route::Home {}, "Home" }
                        NavLink { to: Route::SlideLeft {}, "Slide L" }
                        NavLink { to: Route::SlideRight {}, "Slide R" }
                        NavLink { to: Route::SlideUp {}, "Slide Up" }
                        NavLink { to: Route::SlideDown {}, "Slide Down" }
                        NavLink { to: Route::Fade {}, "Fade" }
                        NavLink { to: Route::Scale {}, "Scale" }
                    }
                }
            }
            AnimatedOutlet {}
        }
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
fn TransitionPage(title: &'static str, description: &'static str, bg_color: &'static str, accent: &'static str) -> Element {
    rsx! {
        div { class: "min-h-screen pt-16 {bg_color}",
            div { class: "max-w-4xl mx-auto p-8",
                div { class: "bg-white/80 backdrop-blur-md rounded-2xl shadow-xl p-8 border border-gray-100",
                    h1 { class: "text-4xl font-bold mb-4 {accent}", "{title}" }
                    p { class: "text-gray-600 text-lg", "{description}" }
                    div { class: "mt-8 grid grid-cols-3 gap-4",
                        {
                            (1..=3)
                                .map(|i| {
                                    rsx! {
                                        div { class: "bg-white p-4 rounded-lg shadow border border-gray-100",
                                            div { class: "w-full h-32 rounded-md {bg_color} mb-4" }
                                            h3 { class: "font-medium mb-2", "Card {i}" }
                                            p { class: "text-sm text-gray-500", "Sample content for card {i}" }
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        TransitionPage {
            title: "Welcome",
            description: "Explore different transition animations between routes",
            bg_color: "bg-gradient-to-br from-blue-50 to-indigo-100",
            accent: "text-blue-600",
        }
    }
}

#[component]
fn SlideLeft() -> Element {
    rsx! {
        TransitionPage {
            title: "Slide Left",
            description: "Smooth horizontal sliding transition from right to left",
            bg_color: "bg-gradient-to-br from-red-50 to-pink-100",
            accent: "text-red-600",
        }
    }
}

#[component]
fn SlideRight() -> Element {
    rsx! {
        TransitionPage {
            title: "Slide Right",
            description: "Elegant horizontal sliding transition from left to right",
            bg_color: "bg-gradient-to-br from-green-50 to-emerald-100",
            accent: "text-emerald-600",
        }
    }
}

#[component]
fn SlideUp() -> Element {
    rsx! {
        TransitionPage {
            title: "Slide Up",
            description: "Vertical sliding transition moving upwards",
            bg_color: "bg-gradient-to-br from-blue-50 to-cyan-100",
            accent: "text-cyan-600",
        }
    }
}

#[component]
fn SlideDown() -> Element {
    rsx! {
        TransitionPage {
            title: "Slide Down",
            description: "Vertical sliding transition moving downwards",
            bg_color: "bg-gradient-to-br from-orange-50 to-amber-100",
            accent: "text-amber-600",
        }
    }
}

#[component]
fn Fade() -> Element {
    rsx! {
        TransitionPage {
            title: "Fade",
            description: "Smooth opacity transition between routes",
            bg_color: "bg-gradient-to-br from-purple-50 to-fuchsia-100",
            accent: "text-fuchsia-600",
        }
    }
}

#[component]
fn Scale() -> Element {
    rsx! {
        TransitionPage {
            title: "Scale",
            description: "Zoom transition with smooth scaling effect",
            bg_color: "bg-gradient-to-br from-rose-50 to-red-100",
            accent: "text-rose-600",
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
