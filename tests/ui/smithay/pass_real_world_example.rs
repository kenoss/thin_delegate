// Compare with https://github.com/Smithay/smithay/blob/8e49b9b/anvil/src/focus.rs

use smithay::input::Seat;

mod __external_trait_def {
    pub(crate) mod smithay {
        #[thin_delegate::external_trait_def]
        pub(crate) mod utils {
            #[thin_delegate::register]
            pub trait IsAlive {
                /// Check if object is alive
                fn alive(&self) -> bool;
            }
        }

        pub(crate) mod input {
            #[thin_delegate::external_trait_def(with_uses = true)]
            pub(crate) mod keyboard {
                use smithay::backend::input::KeyState;
                use smithay::input::keyboard::{KeysymHandle, ModifiersState};
                use smithay::input::SeatHandler;
                use smithay::utils::Serial;

                #[thin_delegate::register]
                pub trait KeyboardTarget<D>:
                    IsAlive + PartialEq + Clone + fmt::Debug + Send
                where
                    D: SeatHandler,
                {
                    /// Keyboard focus of a given seat was assigned to this handler
                    fn enter(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        keys: Vec<KeysymHandle<'_>>,
                        serial: Serial,
                    );
                    /// The keyboard focus of a given seat left this handler
                    fn leave(&self, seat: &Seat<D>, data: &mut D, serial: Serial);
                    /// A key was pressed on a keyboard from a given seat
                    fn key(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        key: KeysymHandle<'_>,
                        state: KeyState,
                        serial: Serial,
                        time: u32,
                    );
                    /// Hold modifiers were changed on a keyboard from a given seat
                    fn modifiers(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        modifiers: ModifiersState,
                        serial: Serial,
                    );
                }
            }

            #[thin_delegate::external_trait_def(with_uses = true)]
            pub(crate) mod pointer {
                use smithay::input::pointer::{
                    AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent,
                    GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent,
                    GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent,
                    MotionEvent, RelativeMotionEvent,
                };
                use smithay::input::SeatHandler;
                use smithay::utils::{IsAlive, Serial};

                #[thin_delegate::register]
                pub trait PointerTarget<D>:
                    IsAlive + PartialEq + Clone + fmt::Debug + Send
                where
                    D: SeatHandler,
                {
                    /// A pointer of a given seat entered this handler
                    fn enter(&self, seat: &Seat<D>, data: &mut D, event: &MotionEvent);
                    /// A pointer of a given seat moved over this handler
                    fn motion(&self, seat: &Seat<D>, data: &mut D, event: &MotionEvent);
                    /// A pointer of a given seat that provides relative motion moved over this handler
                    fn relative_motion(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &RelativeMotionEvent,
                    );
                    /// A pointer of a given seat clicked a button
                    fn button(&self, seat: &Seat<D>, data: &mut D, event: &ButtonEvent);
                    /// A pointer of a given seat scrolled on an axis
                    fn axis(&self, seat: &Seat<D>, data: &mut D, frame: AxisFrame);
                    /// End of a pointer frame
                    fn frame(&self, seat: &Seat<D>, data: &mut D);
                    /// A pointer of a given seat started a swipe gesture
                    fn gesture_swipe_begin(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &GestureSwipeBeginEvent,
                    );
                    /// A pointer of a given seat updated a swipe gesture
                    fn gesture_swipe_update(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &GestureSwipeUpdateEvent,
                    );
                    /// A pointer of a given seat ended a swipe gesture
                    fn gesture_swipe_end(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &GestureSwipeEndEvent,
                    );
                    /// A pointer of a given seat started a pinch gesture
                    fn gesture_pinch_begin(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &GesturePinchBeginEvent,
                    );
                    /// A pointer of a given seat updated a pinch gesture
                    fn gesture_pinch_update(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &GesturePinchUpdateEvent,
                    );
                    /// A pointer of a given seat ended a pinch gesture
                    fn gesture_pinch_end(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &GesturePinchEndEvent,
                    );
                    /// A pointer of a given seat started a hold gesture
                    fn gesture_hold_begin(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &GestureHoldBeginEvent,
                    );
                    /// A pointer of a given seat ended a hold gesture
                    fn gesture_hold_end(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &GestureHoldEndEvent,
                    );
                    /// A pointer of a given seat left this handler
                    fn leave(&self, seat: &Seat<D>, data: &mut D, serial: Serial, time: u32);
                    /// A pointer of a given seat moved from another handler to this handler
                    fn replace(
                        &self,
                        replaced: <D as SeatHandler>::PointerFocus,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &MotionEvent,
                    ) {
                        PointerTarget::<D>::leave(&replaced, seat, data, event.serial, event.time);
                        PointerTarget::<D>::enter(self, seat, data, event);
                    }
                }
            }

            #[thin_delegate::external_trait_def(with_uses = true)]
            pub(crate) mod touch {
                use smithay::input::touch::{
                    DownEvent, MotionEvent, OrientationEvent, ShapeEvent, UpEvent,
                };
                use smithay::input::SeatHandler;
                use smithay::utils::Serial;

                #[thin_delegate::register]
                pub trait TouchTarget<D>: IsAlive + PartialEq + Clone + fmt::Debug + Send
                where
                    D: SeatHandler,
                {
                    /// A new touch point has appeared on the target.
                    ///
                    /// This touch point is assigned a unique ID. Future events from this touch point reference this ID.
                    /// The ID ceases to be valid after a touch up event and may be reused in the future.
                    fn down(&self, seat: &Seat<D>, data: &mut D, event: &DownEvent, seq: Serial);

                    /// The touch point has disappeared.
                    ///
                    /// No further events will be sent for this touch point and the touch point's ID
                    /// is released and may be reused in a future touch down event.
                    fn up(&self, seat: &Seat<D>, data: &mut D, event: &UpEvent, seq: Serial);

                    /// A touch point has changed coordinates.
                    fn motion(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &MotionEvent,
                        seq: Serial,
                    );

                    /// Indicates the end of a set of events that logically belong together.
                    fn frame(&self, seat: &Seat<D>, data: &mut D, seq: Serial);

                    /// Touch session cancelled.
                    ///
                    /// Touch cancellation applies to all touch points currently active on this target.
                    /// The client is responsible for finalizing the touch points, future touch points on
                    /// this target may reuse the touch point ID.
                    fn cancel(&self, seat: &Seat<D>, data: &mut D, seq: Serial);

                    /// Sent when a touch point has changed its shape.
                    ///
                    /// A touch point shape is approximated by an ellipse through the major and minor axis length.
                    /// The major axis length describes the longer diameter of the ellipse, while the minor axis
                    /// length describes the shorter diameter. Major and minor are orthogonal and both are specified
                    /// in surface-local coordinates. The center of the ellipse is always at the touch point location
                    /// as reported by [`TouchTarget::down`] or [`TouchTarget::motion`].
                    fn shape(&self, seat: &Seat<D>, data: &mut D, event: &ShapeEvent, seq: Serial);

                    /// Sent when a touch point has changed its orientation.
                    ///
                    /// The orientation describes the clockwise angle of a touch point's major axis to the positive surface
                    /// y-axis and is normalized to the -180 to +180 degree range. The granularity of orientation depends
                    /// on the touch device, some devices only support binary rotation values between 0 and 90 degrees.
                    fn orientation(
                        &self,
                        seat: &Seat<D>,
                        data: &mut D,
                        event: &OrientationEvent,
                        seq: Serial,
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::register]
struct Window(smithay::desktop::Window);

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def::smithay::utils)]
impl smithay::utils::IsAlive for Window {}

#[thin_delegate::fill_delegate(
    external_trait_def = __external_trait_def::smithay::input::keyboard,
    scheme = |f| {
        match self.0.underlying_surface() {
            smithay::desktop::WindowSurface::Wayland(s) => f(s.wl_surface()),
            smithay::desktop::WindowSurface::X11(s) => f(s),
        }
    }
)]
impl smithay::input::keyboard::KeyboardTarget<State> for Window {}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::register]
enum KeyboardFocusTarget {
    Window(Window),
    LayerSurface(smithay::desktop::LayerSurface),
    Popup(smithay::desktop::PopupKind),
}

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def::smithay::utils)]
impl smithay::utils::IsAlive for KeyboardFocusTarget {}

#[thin_delegate::fill_delegate(
    external_trait_def = __external_trait_def::smithay::input::keyboard,
    scheme = |f| {
        match self {
            Self::Window(w) => f(w),
            Self::LayerSurface(s) => f(s.wl_surface()),
            Self::Popup(p) => f(p.wl_surface()),
        }
    }
)]
impl smithay::input::keyboard::KeyboardTarget<State> for KeyboardFocusTarget {}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::register]
enum PointerFocusTarget {
    WlSurface(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface),
    X11Surface(smithay::xwayland::X11Surface),
}

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def::smithay::utils)]
impl smithay::utils::IsAlive for PointerFocusTarget {}

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def::smithay::input::pointer)]
impl smithay::input::pointer::PointerTarget<State> for PointerFocusTarget {}

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def::smithay::input::touch)]
impl smithay::input::touch::TouchTarget<State> for PointerFocusTarget {}

struct State;

#[allow(unused)]
impl smithay::input::SeatHandler for State {
    type KeyboardFocus = KeyboardFocusTarget;
    type PointerFocus = PointerFocusTarget;
    type TouchFocus = PointerFocusTarget;

    fn seat_state(&mut self) -> &mut smithay::input::SeatState<State> {
        todo!();
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, target: Option<&KeyboardFocusTarget>) {
        todo!();
    }
    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        image: smithay::input::pointer::CursorImageStatus,
    ) {
        todo!();
    }

    fn led_state_changed(
        &mut self,
        _seat: &Seat<Self>,
        led_state: smithay::input::keyboard::LedState,
    ) {
        todo!();
    }
}

fn main() {}
