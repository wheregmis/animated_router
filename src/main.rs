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
    let mut transform = use_motion(Transform::identity());
    let mut opacity = use_motion(1.0f32);

    use_effect(move || {
        transform.animate_to(
            transition.get_transform(),
            AnimationConfig::new(AnimationMode::Tween(Tween::new(Duration::from_millis(500)))),
        );
        opacity.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::new(Duration::from_millis(500)))),
        );
    });

    use_effect(move || {
        if !transform.is_running() {
            animated_router.write().settle();
        }
    });

    rsx! {
        div {
            class: "route-container",
            style: "height: 100%; width: 100%; position: relative;
                   transform: translate({transform.get_value().x}px, {transform.get_value().y}px) 
                   scale({transform.get_value().scale});
                   opacity: {opacity.get_value()};
                   ",
            {from}
            Outlet::<Route> {}
        }
    }
}

#[component]
fn Expand(children: Element) -> Element {
    rsx! {
        div {
            class: "expand",
            style: "height: 100%; width: 100%;
                   display: flex; align-items: center; justify-content: center;",
            {children}
        }
    }
}

#[component]
fn AnimatedOutlet(children: Element) -> Element {
    let animated_router = use_context::<Signal<AnimatedRouterContext<Route>>>();

    let from_route = match animated_router() {
        AnimatedRouterContext::FromTo(Route::Home {}, Route::Blog {}) => {
            Some((rsx!(Home {}), TransitionVariant::SlideRight))
        }
        AnimatedRouterContext::FromTo(Route::Blog {}, Route::Home {}) => {
            Some((rsx!(Blog {}), TransitionVariant::SlideLeft))
        }
        _ => None,
    };

    rsx! {
        div {
            if let Some((from, transition)) = from_route {
                FromRouteToCurrent { from, transition }
            } else {
                Expand { Outlet::<Route> {} }
            }
        }
    }
}

// Turn off rustfmt since we're doing layouts and routes in the same enum
#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
#[allow(clippy::empty_line_after_outer_attr)]
enum Route {
    // Wrap Home in a Navbar Layout
    #[layout(NavBar)]
        // The default route is always "/" unless otherwise specified
        #[route("/")]
        Home {},

        #[route("/blog")]
        Blog {},

    // And the regular page layout
    #[end_layout]



    // Finally, we need to handle the 404 page
    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}

#[component]
fn NavBar() -> Element {
    rsx! {
        AnimatedRouter::<Route> {
            nav { id: "navbar",
                Link { to: Route::Home {}, "Home" }
                Link { to: Route::Blog {}, "Blog" }
            }
            AnimatedOutlet {}
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div { style: "background-color: #f0f0f0; width: 100%; height: 100vh; padding: 2rem;",
            div { style: "background-color: white; padding: 2rem; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);",
                h1 { "Welcome to the Dioxus Blog!" }
                p { "This is a demonstration of smooth route transitions." }
            }
        }
    }
}

#[component]
fn Blog() -> Element {
    rsx! {
        div { style: "background-color: #e0e0ff; width: 100%; height: 100vh; padding: 2rem;",
            div { style: "background-color: white; padding: 2rem; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);",
                h1 { "Blog" }
                p { "Welcome to our blog section!" }
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

fn main() {
    dioxus::launch(|| {
        rsx! {
            Router::<Route> {}
        }
    });
}
