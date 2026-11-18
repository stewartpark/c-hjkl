use libc::{c_char, c_void, input_event, ioctl, open, read, O_RDONLY};

const EVIOCGRAB: u64 = 1074021776;

// From linux/input-event-codes.h
const EV_KEY: u16 = 0x01;
const KEY_LEFTCTRL: u16 = 29;
const KEY_CAPSLOCK: u16 = 58;
const KEY_H: u16 = 35;
const KEY_J: u16 = 36;
const KEY_K: u16 = 37;
const KEY_L: u16 = 38;
const KEY_UP: u16 = 103;
const KEY_LEFT: u16 = 105;
const KEY_RIGHT: u16 = 106;
const KEY_DOWN: u16 = 108;

/// Rewrite the Caps Lock key code to Left Ctrl. All other codes are unchanged.
fn rewrite_code(code: u16) -> u16 {
    if code == KEY_CAPSLOCK {
        KEY_LEFTCTRL
    } else {
        code
    }
}

/// Map a Ctrl+HJKL key press to the corresponding arrow key.
///
/// Returns `Some(events)` when the input event must be replaced by exactly
/// those `(code, value)` events (the original event is swallowed), or
/// `None` when the input event should pass through unchanged.
fn remap_ctrl_combo(code: u16, value: i32, ctrl_pressed: bool) -> Option<Vec<(u16, i32)>> {
    if !ctrl_pressed || value < 1 {
        return None;
    }

    let key_to_press = if code == KEY_H {
        KEY_LEFT
    } else if code == KEY_J {
        KEY_DOWN
    } else if code == KEY_K {
        KEY_UP
    } else if code == KEY_L {
        KEY_RIGHT
    } else {
        return None;
    };

    Some(vec![
        (KEY_LEFTCTRL, 0),
        (key_to_press, 1),
        (key_to_press, 0),
        (KEY_LEFTCTRL, 1),
    ])
}

pub struct KeyboardHandler {
    fd: i32,
    uinput: uinput::Device,
    is_grabbed: bool,
    debug: bool,
    device_path: String,
}

impl KeyboardHandler {
    pub fn new(device_path: &String, debug: bool) -> KeyboardHandler {
        unsafe {
            let fd = open(device_path[..].as_ptr() as *const c_char, O_RDONLY);
            if fd == -1 {
                panic!("Cannot open input device: {}", device_path);
            }

            KeyboardHandler {
                device_path: device_path.to_string(),
                is_grabbed: false,
                uinput: uinput::default()
                    .unwrap()
                    .name(format!("C-HJKL Output for {}", device_path))
                    .unwrap()
                    .event(uinput::event::Keyboard::All)
                    .unwrap()
                    .create()
                    .unwrap(),
                debug,
                fd,
            }
        }
    }

    fn grab(&mut self) {
        unsafe {
            if !self.is_grabbed && ioctl(self.fd, EVIOCGRAB, 1) != -1 {
                self.is_grabbed = true;
            }
        }
    }

    #[allow(dead_code)]
    fn ungrab(&mut self) {
        unsafe {
            ioctl(self.fd, EVIOCGRAB, 0);
            self.is_grabbed = false;
        }
    }

    fn read(&self) -> input_event {
        unsafe {
            let mut ev: input_event = std::mem::zeroed();
            if read(
                self.fd,
                &mut ev as *mut _ as *mut c_void,
                std::mem::size_of::<input_event>(),
            ) != (std::mem::size_of::<input_event>() as _)
            {
                panic!("Read a partial event");
            }
            ev.clone()
        }
    }

    fn write(&mut self, ev: &input_event) {
        self.uinput
            .write(ev.type_ as _, ev.code as _, ev.value)
            .unwrap();
    }

    pub fn run_forever(&mut self) {
        let mut ctrl_pressed = false;

        std::thread::sleep(std::time::Duration::from_secs(1));

        self.grab();
        loop {
            let mut input = self.read();

            if self.debug {
                println!(
                    "[{}] ctrl:{}, ev: {} {} {}",
                    self.device_path, ctrl_pressed, input.type_, input.code, input.value
                );
            }

            if input.type_ == EV_KEY {
                // Handle Capslock / Ctrl
                input.code = rewrite_code(input.code);

                // Maintain Ctrl flag
                if input.code == KEY_LEFTCTRL {
                    ctrl_pressed = input.value != 0;
                }

                // Handle C-hjkl
                if let Some(events) = remap_ctrl_combo(input.code, input.value, ctrl_pressed) {
                    for (code, value) in events {
                        input.code = code;
                        input.value = value;
                        self.write(&input);
                    }
                    continue;
                }
            }

            // Pass-through
            self.write(&input);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capslock_code_is_rewritten_to_left_ctrl() {
        assert_eq!(rewrite_code(KEY_CAPSLOCK), KEY_LEFTCTRL);
    }

    #[test]
    fn non_capslock_codes_are_not_rewritten() {
        for code in [KEY_H, KEY_J, KEY_K, KEY_L, KEY_UP, KEY_LEFT, KEY_LEFTCTRL] {
            assert_eq!(rewrite_code(code), code);
        }
    }

    #[test]
    fn ctrl_h_remaps_to_left_arrow() {
        assert_eq!(
            remap_ctrl_combo(KEY_H, 1, true),
            Some(vec![
                (KEY_LEFTCTRL, 0),
                (KEY_LEFT, 1),
                (KEY_LEFT, 0),
                (KEY_LEFTCTRL, 1),
            ])
        );
    }

    #[test]
    fn ctrl_jkl_remap_to_down_up_right_arrows() {
        assert_eq!(
            remap_ctrl_combo(KEY_J, 1, true),
            Some(vec![
                (KEY_LEFTCTRL, 0),
                (KEY_DOWN, 1),
                (KEY_DOWN, 0),
                (KEY_LEFTCTRL, 1),
            ])
        );
        assert_eq!(
            remap_ctrl_combo(KEY_K, 1, true),
            Some(vec![
                (KEY_LEFTCTRL, 0),
                (KEY_UP, 1),
                (KEY_UP, 0),
                (KEY_LEFTCTRL, 1),
            ])
        );
        assert_eq!(
            remap_ctrl_combo(KEY_L, 1, true),
            Some(vec![
                (KEY_LEFTCTRL, 0),
                (KEY_RIGHT, 1),
                (KEY_RIGHT, 0),
                (KEY_LEFTCTRL, 1),
            ])
        );
    }

    #[test]
    fn no_remap_without_ctrl_pressed() {
        for code in [KEY_H, KEY_J, KEY_K, KEY_L] {
            assert_eq!(remap_ctrl_combo(code, 1, false), None);
        }
    }

    #[test]
    fn no_remap_on_key_release() {
        for code in [KEY_H, KEY_J, KEY_K, KEY_L] {
            assert_eq!(remap_ctrl_combo(code, 0, true), None);
        }
    }

    #[test]
    fn no_remap_for_keys_outside_hjkl() {
        for code in [KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, 1, 255] {
            assert_eq!(remap_ctrl_combo(code, 1, true), None);
        }
    }
}
