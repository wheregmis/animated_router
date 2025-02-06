use dioxus::prelude::*;
use dioxus::router::prelude::{use_route, Routable};
use dioxus_motion::prelude::*;

use crate::Route;

/// Ask for MARC Permission and Give him Credit for his work on this code

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
pub fn AnimatedOutlet(children: Element) -> Element {
    let animated_router = use_context::<Signal<AnimatedRouterContext<Route>>>();
    let from_route: Option<(Result<VNode, RenderError>, TransitionVariant)> =
        match animated_router() {
            AnimatedRouterContext::FromTo(from, to) => {
                Some((from.get_component(), to.get_transition()))
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

#[component]
pub fn AnimationBuilder() -> Element {
    rsx! {
        AnimatedRouter::<Route> { AnimatedOutlet {} }
    }
}
