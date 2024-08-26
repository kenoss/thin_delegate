// Compare with https://github.com/Smithay/smithay/blob/8e49b9b/anvil/src/focus.rs

use smithay::backend::input::KeyState;
use smithay::input::keyboard::{KeyboardTarget, KeysymHandle, ModifiersState};
use smithay::input::pointer::{AxisFrame, ButtonEvent, MotionEvent, RelativeMotionEvent};
use smithay::input::pointer::{
    GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent, GesturePinchEndEvent,
    GesturePinchUpdateEvent, GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent,
};
use smithay::input::touch::{DownEvent, OrientationEvent, ShapeEvent, UpEvent};
use smithay::input::Seat;
use smithay::input::SeatHandler;
use smithay::utils::Serial;

mod private_for_thin_delegate {
    #[thin_delegate::register(smithay::utils::IsAlive)]
    pub trait IsAlive {
        /// Check if object is alive
        fn alive(&self) -> bool;
    }

    #[thin_delegate::register(smithay::input::keyboard::KeyboardTarget)]
    pub trait KeyboardTarget<D>: IsAlive + PartialEq + Clone + fmt::Debug + Send
    where
        D: SeatHandler,
    {
        /// Keyboard focus of a given seat was assigned to this handler
        fn enter(&self, seat: &Seat<D>, data: &mut D, keys: Vec<KeysymHandle<'_>>, serial: Serial);
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

    #[thin_delegate::register(smithay::input::pointer::PointerTarget)]
    pub trait PointerTarget<D>: IsAlive + PartialEq + Clone + fmt::Debug + Send
    where
        D: SeatHandler,
    {
        /// A pointer of a given seat entered this handler
        fn enter(&self, seat: &Seat<D>, data: &mut D, event: &MotionEvent);
        /// A pointer of a given seat moved over this handler
        fn motion(&self, seat: &Seat<D>, data: &mut D, event: &MotionEvent);
        /// A pointer of a given seat that provides relative motion moved over this handler
        fn relative_motion(&self, seat: &Seat<D>, data: &mut D, event: &RelativeMotionEvent);
        /// A pointer of a given seat clicked a button
        fn button(&self, seat: &Seat<D>, data: &mut D, event: &ButtonEvent);
        /// A pointer of a given seat scrolled on an axis
        fn axis(&self, seat: &Seat<D>, data: &mut D, frame: AxisFrame);
        /// End of a pointer frame
        fn frame(&self, seat: &Seat<D>, data: &mut D);
        /// A pointer of a given seat started a swipe gesture
        fn gesture_swipe_begin(&self, seat: &Seat<D>, data: &mut D, event: &GestureSwipeBeginEvent);
        /// A pointer of a given seat updated a swipe gesture
        fn gesture_swipe_update(
            &self,
            seat: &Seat<D>,
            data: &mut D,
            event: &GestureSwipeUpdateEvent,
        );
        /// A pointer of a given seat ended a swipe gesture
        fn gesture_swipe_end(&self, seat: &Seat<D>, data: &mut D, event: &GestureSwipeEndEvent);
        /// A pointer of a given seat started a pinch gesture
        fn gesture_pinch_begin(&self, seat: &Seat<D>, data: &mut D, event: &GesturePinchBeginEvent);
        /// A pointer of a given seat updated a pinch gesture
        fn gesture_pinch_update(
            &self,
            seat: &Seat<D>,
            data: &mut D,
            event: &GesturePinchUpdateEvent,
        );
        /// A pointer of a given seat ended a pinch gesture
        fn gesture_pinch_end(&self, seat: &Seat<D>, data: &mut D, event: &GesturePinchEndEvent);
        /// A pointer of a given seat started a hold gesture
        fn gesture_hold_begin(&self, seat: &Seat<D>, data: &mut D, event: &GestureHoldBeginEvent);
        /// A pointer of a given seat ended a hold gesture
        fn gesture_hold_end(&self, seat: &Seat<D>, data: &mut D, event: &GestureHoldEndEvent);
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
            data.cursor_image(seat, CursorImageStatus::default_named());
            PointerTarget::<D>::enter(self, seat, data, event);
        }
    }

    #[thin_delegate::register(smithay::input::touch::TouchTarget)]
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
        // fn motion(&self, seat: &Seat<D>, data: &mut D, event: &MotionEvent, seq: Serial);
        fn motion(
            &self,
            seat: &Seat<D>,
            data: &mut D,
            event: &smithay::input::touch::MotionEvent,
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
        fn orientation(&self, seat: &Seat<D>, data: &mut D, event: &OrientationEvent, seq: Serial);
    }
}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::derive_delegate(smithay::utils::IsAlive)]
struct Window(smithay::desktop::Window);

impl smithay::input::keyboard::KeyboardTarget<State> for Window {
    fn enter(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        keys: Vec<KeysymHandle<'_>>,
        serial: Serial,
    ) {
        match self.0.underlying_surface() {
            smithay::desktop::WindowSurface::Wayland(w) => {
                KeyboardTarget::enter(w.wl_surface(), seat, data, keys, serial)
            }

            smithay::desktop::WindowSurface::X11(s) => {
                KeyboardTarget::enter(s, seat, data, keys, serial)
            }
        }
    }

    fn leave(&self, seat: &Seat<State>, data: &mut State, serial: Serial) {
        match self.0.underlying_surface() {
            smithay::desktop::WindowSurface::Wayland(w) => {
                KeyboardTarget::leave(w.wl_surface(), seat, data, serial)
            }

            smithay::desktop::WindowSurface::X11(s) => KeyboardTarget::leave(s, seat, data, serial),
        }
    }

    fn key(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        key: KeysymHandle<'_>,
        state: KeyState,
        serial: Serial,
        time: u32,
    ) {
        match self.0.underlying_surface() {
            smithay::desktop::WindowSurface::Wayland(w) => {
                KeyboardTarget::key(w.wl_surface(), seat, data, key, state, serial, time)
            }

            smithay::desktop::WindowSurface::X11(s) => {
                KeyboardTarget::key(s, seat, data, key, state, serial, time)
            }
        }
    }

    fn modifiers(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        modifiers: ModifiersState,
        serial: Serial,
    ) {
        match self.0.underlying_surface() {
            smithay::desktop::WindowSurface::Wayland(w) => {
                KeyboardTarget::modifiers(w.wl_surface(), seat, data, modifiers, serial)
            }

            smithay::desktop::WindowSurface::X11(s) => {
                KeyboardTarget::modifiers(s, seat, data, modifiers, serial)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::derive_delegate(
    smithay::utils::IsAlive,
    smithay::input::keyboard::KeyboardTarget<State>,
)]
enum KeyboardFocusTarget {
    Window(Window),
    #[delegate_to(x => x.wl_surface())]
    LayerSurface(smithay::desktop::LayerSurface),
    #[delegate_to(x => x.wl_surface())]
    Popup(smithay::desktop::PopupKind),
}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::derive_delegate(
    smithay::utils::IsAlive,
    smithay::input::pointer::PointerTarget<State>,
    smithay::input::touch::TouchTarget<State>,
)]
enum PointerFocusTarget {
    WlSurface(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface),
    X11Surface(smithay::xwayland::X11Surface),
}

struct State;

#[allow(unused)]
impl SeatHandler for State {
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
