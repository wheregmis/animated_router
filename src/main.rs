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

    rsx!({ children })
}

/// Shortcut to get access to the [AnimatedRouterContext].
pub fn use_animated_router<Route: Routable + PartialEq>() -> Signal<AnimatedRouterContext<Route>> {
    use_context()
}

#[derive(Clone)]
pub struct TransitionConfig {
    initial_from: Transform,
    final_from: Transform,
    initial_to: Transform,
    final_to: Transform,
}

#[derive(PartialEq, Clone)]
pub enum TransitionVariant {
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Fade,
}

impl TransitionVariant {
    fn get_config(&self) -> TransitionConfig {
        match self {
            TransitionVariant::SlideLeft => TransitionConfig {
                initial_from: Transform::identity(),
                final_from: Transform::new(-100.0, 0.0, 1.0, 1.0),
                initial_to: Transform::new(100.0, 0.0, 1.0, 1.0),
                final_to: Transform::identity(),
            },
            TransitionVariant::SlideRight => TransitionConfig {
                initial_from: Transform::identity(),
                final_from: Transform::new(100.0, 0.0, 1.0, 1.0),
                initial_to: Transform::new(-100.0, 0.0, 1.0, 1.0),
                final_to: Transform::identity(),
            },
            TransitionVariant::SlideUp => TransitionConfig {
                initial_from: Transform::identity(),
                final_from: Transform::new(0.0, -100.0, 1.0, 1.0),
                initial_to: Transform::new(0.0, 100.0, 1.0, 1.0),
                final_to: Transform::identity(),
            },
            TransitionVariant::SlideDown => TransitionConfig {
                initial_from: Transform::identity(),
                final_from: Transform::new(0.0, 100.0, 1.0, 1.0),
                initial_to: Transform::new(0.0, -100.0, 1.0, 1.0),
                final_to: Transform::identity(),
            },
            TransitionVariant::Fade => TransitionConfig {
                initial_from: Transform::new(0.0, 0.0, 1.0, 1.0),
                final_from: Transform::new(0.0, 0.0, 1.0, 0.0),
                initial_to: Transform::new(0.0, 0.0, 1.0, 0.0),
                final_to: Transform::new(0.0, 0.0, 1.0, 1.0),
            },
        }
    }
}
#[component]
fn FromRouteToCurrent(from: Element, transition: TransitionVariant) -> Element {
    let mut animated_router = use_animated_router::<Route>();
    let config = transition.get_config();
    let mut from_transform = use_motion(config.initial_from);
    let mut to_transform = use_motion(config.initial_to);
    let mut from_opacity = use_motion(1.0f32);
    let mut to_opacity = use_motion(0.0f32);

    use_effect(move || {
        let spring = Spring {
            stiffness: 160.0, // Reduced from 180.0 for less aggressive movement
            damping: 20.0,    // Increased from 12.0 for faster settling
            mass: 1.5,        // Slightly increased for more "weight"
            velocity: 10.0,   // Keep at 0 for predictable start
        };

        // Animate FROM route
        from_transform.animate_to(
            config.final_from,
            AnimationConfig::new(AnimationMode::Spring(spring)),
        );

        // Animate TO route
        to_transform.animate_to(
            config.final_to,
            AnimationConfig::new(AnimationMode::Spring(spring)),
        );

        // Fade out old route
        from_opacity.animate_to(0.0, AnimationConfig::new(AnimationMode::Spring(spring)));
        to_opacity.animate_to(1.0, AnimationConfig::new(AnimationMode::Spring(spring)));
    });

    use_effect(move || {
        if !from_transform.is_running() && !to_transform.is_running() {
            animated_router.write().settle();
        }
    });

    rsx! {
        div {
            class: "route-container",
            style: "
                position: relative; 
                width: 100%; 
                height: 100vh; 
                overflow: hidden;
                transform-style: preserve-3d;
                -webkit-transform-style: preserve-3d;
                -webkit-tap-highlight-color: transparent;
            ",
            div {
                class: "route-content from",
                style: "
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    transform: translate3d({from_transform.get_value().x}%, {from_transform.get_value().y}%, 0) 
                             scale({from_transform.get_value().scale});
                    opacity: {from_opacity.get_value()};
                    will-change: transform, opacity;
                    backface-visibility: hidden;
                    -webkit-backface-visibility: hidden;
                ",
                {from}
            }
            div {
                class: "route-content to",
                style: "
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    transform: translate3d({to_transform.get_value().x}%, {to_transform.get_value().y}%, 0) 
                             scale({to_transform.get_value().scale});
                    opacity: {to_opacity.get_value()};
                    will-change: transform, opacity;
                    backface-visibility: hidden;
                    -webkit-backface-visibility: hidden;
                ",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
fn AnimatedOutlet(children: Element) -> Element {
    let animated_router = use_context::<Signal<AnimatedRouterContext<Route>>>();

    let from_route: Option<(Result<VNode, RenderError>, TransitionVariant)> =
        match animated_router() {
            AnimatedRouterContext::FromTo(from, to) => {
                // Generate component based on route type
                let component = match from {
                    Route::Home {} => Home,
                    Route::SlideLeft {} => SlideLeft,
                    Route::SlideRight {} => SlideRight,
                    Route::SlideUp {} => SlideUp,
                    Route::SlideDown {} => SlideDown,
                    Route::Fade {} => Fade,
                    _ => Home,
                };
                Some((component(), to.transition_variant()))
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
    #[layout(AnimationBuilder)]
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
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

impl Route {
    fn transition_variant(&self) -> TransitionVariant {
        match self {
            Route::SlideLeft {} => TransitionVariant::SlideLeft,
            Route::SlideRight {} => TransitionVariant::SlideRight,
            Route::SlideUp {} => TransitionVariant::SlideUp,
            Route::SlideDown {} => TransitionVariant::SlideDown,
            Route::Fade {} => TransitionVariant::Fade,
            _ => TransitionVariant::SlideRight,
        }
    }
}

#[component]
fn AnimationBuilder() -> Element {
    rsx! {
        AnimatedRouter::<Route> { AnimatedOutlet {} }
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
