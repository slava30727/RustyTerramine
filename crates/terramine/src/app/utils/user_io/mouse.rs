use {
    super::KeyState,
    crate::{prelude::*, window::Window},
    winit::event::MouseButton as WinitMouseButton,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
assert_impl_all!(MouseButton: Send, Sync);

impl ConstDefault for MouseButton {
    const DEFAULT: Self = Self::Left;
}

impl Default for MouseButton {
    fn default() -> Self {
        const_default()
    }
}

impl TryFrom<WinitMouseButton> for MouseButton {
    type Error = ButtonConversionError;

    fn try_from(value: WinitMouseButton) -> Result<Self, Self::Error> {
        use WinitMouseButton::*;
        Ok(match value {
            Left => Self::Left,
            Right => Self::Right,
            Middle => Self::Middle,
            // FIXME: handle new mouse buttons
            Forward | Back => unimplemented!(),
            Other(id) => return Err(ButtonConversionError::UnknownId { id }),
        })
    }
}

impl TryFrom<ExtendedMouseButton> for MouseButton {
    type Error = ButtonConversionError;

    fn try_from(value: ExtendedMouseButton) -> Result<Self, Self::Error> {
        use ExtendedMouseButton::*;
        Ok(match value {
            Left => Self::Left,
            Right => Self::Right,
            Middle => Self::Middle,
            Other(id) => return Err(ButtonConversionError::UnknownId { id }),
        })
    }
}

impl From<MouseButton> for WinitMouseButton {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => Self::Left,
            MouseButton::Middle => Self::Middle,
            MouseButton::Right => Self::Right,
        }
    }
}

#[derive(Debug, Error)]
pub enum ButtonConversionError {
    #[error("failed to convert mouse button: unknown id: {id}")]
    UnknownId { id: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IsVariant)]
pub enum ExtendedMouseButton {
    Left,
    Middle,
    Right,
    Other(u16),
}
assert_impl_all!(ExtendedMouseButton: Send, Sync);

impl ExtendedMouseButton {
    /// Converts [`ExtendedMouseButton`] to just [`MouseButton`]
    /// by removing [`ExtendedMouseButton::Other`] variant.
    ///
    /// # Safety
    ///
    /// - `self` is not [`ExtendedMouseButton::Other`].
    pub unsafe fn to_mouse_button_unchecked(self) -> MouseButton {
        debug_assert!(!self.is_other());

        match self {
            Self::Left => MouseButton::Left,
            Self::Middle => MouseButton::Middle,
            Self::Right => MouseButton::Right,
            Self::Other(_) => hint::unreachable_unchecked(),
        }
    }

    /// Converts [`ExtendedMouseButton`] to just [`ExtendedMouseButton::Other`]'s
    /// id by removing other variants.
    ///
    /// # Safety
    ///
    /// - `self` is [`ExtendedMouseButton::Other`].
    pub unsafe fn to_other_unchecked(self) -> u16 {
        debug_assert!(self.is_other());

        match self {
            Self::Other(id) => id,
            _ => hint::unreachable_unchecked(),
        }
    }
}

impl From<MouseButton> for ExtendedMouseButton {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => Self::Left,
            MouseButton::Middle => Self::Middle,
            MouseButton::Right => Self::Right,
        }
    }
}

impl From<WinitMouseButton> for ExtendedMouseButton {
    fn from(value: WinitMouseButton) -> Self {
        use WinitMouseButton::*;
        match value {
            Left => Self::Left,
            Right => Self::Right,
            Middle => Self::Middle,
            // FIXME: handle new mouse buttons
            Forward | Back => unimplemented!(),
            Other(id) => Self::Other(id),
        }
    }
}

lazy_static! {
    pub(super) static ref SPECIAL_INPUTS: RwLock<HashSet<u16>> = default();
    pub(super) static ref RELEASED_KEYS: RwLock<HashSet<ExtendedMouseButton>> = default();
}

pub static INPUTS: [Atomic<KeyState>; 3] = const_default();

macros::atomic_static! {
    pub(super) static DX: f32 = 0.0;
    pub(super) static DY: f32 = 0.0;
    pub(super) static X: f32 = 0.0;
    pub(super) static Y: f32 = 0.0;
    pub(super) static IS_ON_WINDOW: bool = false;
    pub(crate) static IS_CAPTURED: bool = false;
}

pub fn get_x() -> f32 {
    X.load(Relaxed)
}
pub fn get_y() -> f32 {
    Y.load(Relaxed)
}
pub fn get_pos() -> Vec2 {
    Vec2::new(get_x(), get_y())
}

pub fn get_dx() -> f32 {
    DX.load(Relaxed)
}
pub fn get_dy() -> f32 {
    DY.load(Relaxed)
}
pub fn get_delta() -> Vec2 {
    Vec2::new(get_dx(), get_dy())
}

pub fn move_cursor(pos: Vec2) {
    let prev_pos = self::get_pos();
    let delta = self::get_delta() + pos - prev_pos;

    macros::store!(Release: DX = delta.x, DY = delta.y);
    macros::store!(Release: X = pos.x, Y = pos.y);
}

pub fn press(button: ExtendedMouseButton) {
    if !button.is_other() {
        let idx = unsafe { button.to_mouse_button_unchecked() } as usize;
        INPUTS[idx].store(KeyState::Pressed, Release);
    } else {
        let id = unsafe { button.to_other_unchecked() };
        SPECIAL_INPUTS.write().insert(id);
    }
}

pub fn release(button: ExtendedMouseButton) {
    if !button.is_other() {
        let idx = unsafe { button.to_mouse_button_unchecked() } as usize;
        INPUTS[idx].store(KeyState::Released, Release);
    } else {
        let id = unsafe { button.to_other_unchecked() };
        SPECIAL_INPUTS.write().remove(&id);
    }
}

pub fn is_pressed_common(button: MouseButton) -> bool {
    INPUTS[button as usize].load(Acquire).is_pressed()
}

pub fn is_pressed(button: ExtendedMouseButton) -> bool {
    if !button.is_other() {
        let button = unsafe { button.to_mouse_button_unchecked() };
        is_pressed_common(button)
    } else {
        let id = unsafe { button.to_other_unchecked() };
        SPECIAL_INPUTS.write().contains(&id)
    }
}

pub fn just_pressed(button: ExtendedMouseButton) -> bool {
    let is_pressed = is_pressed(button);
    if is_pressed {
        RELEASED_KEYS.write().insert(button);
    }
    is_pressed
}

pub fn just_pressed_common(button: MouseButton) -> bool {
    let is_pressed = is_pressed_common(button);
    if is_pressed {
        RELEASED_KEYS.write().insert(button.into());
    }
    is_pressed
}

pub fn is_left_pressed() -> bool {
    is_pressed_common(MouseButton::Left)
}

pub fn is_right_pressed() -> bool {
    is_pressed_common(MouseButton::Right)
}

pub fn is_middle_pressed() -> bool {
    is_pressed_common(MouseButton::Middle)
}

pub fn just_left_pressed() -> bool {
    just_pressed_common(MouseButton::Left)
}

pub fn just_right_pressed() -> bool {
    just_pressed_common(MouseButton::Right)
}

pub fn just_middle_pressed() -> bool {
    just_pressed_common(MouseButton::Middle)
}

pub fn update_cursor_position(window: &Window) -> Result<(), MouseError> {
    // Get cursor position from `winapi`.
    let [x, y] = get_cursor_pos(window)?.to_array();
    let (prev_x, prev_y) = macros::load!(Acquire: X, Y);

    macros::store!(Release: X = x, Y = y, DX = x - prev_x, DY = y - prev_y);

    // Get window size.
    let half_size = window.inner_size().to_vec2() / 2;

    // If mouse is captured then not change mouse position and put cursor on center.
    if IS_CAPTURED.load(Relaxed) {
        window
            .set_cursor_position(half_size.to_physical_position())
            .expect("failed to set cursor position");

        macros::store!(Release: X = half_size.x as f32, Y = half_size.y as f32);
    }

    Ok(())
}

/// Updates the mouse systems.
pub fn update() {
    for key in RELEASED_KEYS.write().drain() {
        release(key);
    }
    macros::store!(Release: DX = 0.0, DY = 0.0);
}

/// Gives cursor position in screen cordinates.
#[cfg(windows)]
pub fn get_cursor_screen_pos() -> Result<IVec2, MouseError> {
    use winapi::{shared::windef::POINT as Point, um::winuser::GetCursorPos as win_get_cursor_pos};

    // Point cordinates struct
    let mut pt: Point = unsafe { mem::zeroed() };

    // Checks if WinAPI `GetCursorPos()` success then return cursor position else error.
    match unsafe { win_get_cursor_pos(&mut pt) } {
        0 => Err(MouseError::GetCursorPos),
        _ => Ok(IVec2::new(pt.x, pt.y)),
    }
}

#[cfg(not(windows))]
pub fn get_cursor_screen_pos() -> Result<IVec2, MouseError> {
    // FIXME(hack3rmann): mouse movement on Wayland
    Ok(IVec2::new(0, 0))
}

/// Gives cursor position in window cordinates.
#[cfg(windows)]
pub fn get_cursor_pos(window: &Window) -> Result<Vec2, MouseError> {
    let [x, y] = get_cursor_screen_pos()?.to_array();
    let window_pos = window.inner_position().unwrap_or_default();

    Ok(IVec2::new(x - window_pos.x, y - window_pos.y).as_vec2())
}

#[cfg(not(windows))]
pub fn get_cursor_pos(_window: &Window) -> Result<Vec2, MouseError> {
    Ok(get_pos())
}

pub fn is_captured() -> bool {
    IS_CAPTURED.load(Acquire)
}

/// Grabs the cursor for camera control.
pub fn capture(window: &Window) {
    use winit::window::CursorGrabMode;

    if IS_CAPTURED.load(Acquire) {
        return;
    }

    // FIXME(hack3rmann): figure out how to make camera controls
    // window.set_cursor_grab(CursorGrabMode::Locked)
    //     .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
    //     .expect("failed to grab the cursor");
    // window.set_cursor_visible(false);

    IS_CAPTURED.store(true, Release);
}

/// Releases cursor for standart input.
pub fn uncapture(window: &Window) {
    use winit::window::CursorGrabMode;

    if !IS_CAPTURED.load(Acquire) {
        return;
    }

    window
        .set_cursor_grab(CursorGrabMode::None)
        .expect("failed to release the cursor");
    window.set_cursor_visible(true);

    IS_CAPTURED.store(false, Release);
}

/// Sets capture mode.
pub fn set_capture(window: &Window, do_capture: bool) {
    if do_capture {
        capture(window);
    } else {
        uncapture(window);
    }
}

#[derive(Debug, Error)]
pub enum MouseError {
    #[error("failed to get cursor position (os error code 0)")]
    GetCursorPos,

    #[error(transparent)]
    NotSupported(#[from] winit::error::NotSupportedError),
}