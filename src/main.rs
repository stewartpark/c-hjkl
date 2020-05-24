use clap::Clap;
use libc::{
    c_char, c_void, gettimeofday, input_event, ioctl, lseek, open, read, timeval, write, O_RDONLY,
    O_RDWR, O_WRONLY, SEEK_CUR,
};
use std::{thread, time};

#[derive(Clap)]
#[clap(version = "0.1", author = "Stewart J. Park <hello@stewartjpark.com>")]
struct Opts {
    #[clap(
        short,
        long,
        default_value = "/dev/input/by-path/platform-i8042-serio-0-event-kbd"
    )]
    device: String,
}

const nilTime: timeval = timeval {
    tv_sec: 0,
    tv_usec: 0,
};

// From linux/input-event-codes.h
const EVIOCGRAB: u64 = 1074021776;
const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const SYN_REPORT: u16 = 0;
const KEY_LEFTCTRL: u16 = 58; //29;
const KEY_H: u16 = 35;
const KEY_J: u16 = 36;
const KEY_K: u16 = 37;
const KEY_L: u16 = 38;
const KEY_UP: u16 = 103;
const KEY_LEFT: u16 = 105;
const KEY_RIGHT: u16 = 106;
const KEY_DOWN: u16 = 108;
const syn: input_event = input_event {
    time: nilTime,
    type_: EV_SYN,
    code: SYN_REPORT,
    value: 0,
};

struct KeyboardHandler {
    fd: i32,
    uinput: uinput::Device,
    is_grabbed: bool,
}

impl KeyboardHandler {
    pub fn new(device_path: &String) -> KeyboardHandler {
        unsafe {
            let fd = open(device_path[..].as_ptr() as *const c_char, O_RDONLY);
            if fd == -1 {
                panic!("Cannot open input device: {}", device_path);
            }

            KeyboardHandler {
                is_grabbed: false,
                uinput: uinput::default()
                    .unwrap()
                    .name("C-HJKL Output")
                    .unwrap()
                    .event(uinput::event::Keyboard::All)
                    .unwrap()
                    .create()
                    .unwrap(),
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

    fn ungrab(&mut self) {
        unsafe {
            ioctl(self.fd, EVIOCGRAB, 0);
            self.is_grabbed = false;
        }
    }

    fn read(&self) -> input_event {
        let mut ev: input_event = unsafe { std::mem::zeroed() };

        unsafe {
            if read(
                self.fd,
                &mut ev as *mut _ as *mut c_void,
                std::mem::size_of::<input_event>(),
            ) != (std::mem::size_of::<input_event>() as _)
            {
                panic!("Read a partial event");
            }
        }
        ev.clone()
    }

    fn write(&mut self, ev: &input_event) {
        self.uinput
            .write(ev.type_ as _, ev.code as _, ev.value)
            .unwrap();
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    let mut handler = KeyboardHandler::new(&opts.device);
    let mut ctrl_pressed = false;

    handler.grab();
    loop {
        let mut input = handler.read();

        // Maintain ctrl flag
        if input.type_ == EV_KEY && input.code == KEY_LEFTCTRL {
            ctrl_pressed = input.value != 0;
        }

        if input.type_ == EV_KEY && input.value >= 1 && ctrl_pressed {
            let key_to_press = if input.code == KEY_H {
                KEY_LEFT
            } else if input.code == KEY_J {
                KEY_DOWN
            } else if input.code == KEY_K {
                KEY_UP
            } else if input.code == KEY_L {
                KEY_RIGHT
            } else {
                0
            };

            if key_to_press > 0 {
                input.value = 0;
                input.code = KEY_LEFTCTRL;
                handler.write(&input);

                input.code = key_to_press;
                input.value = 1;
                handler.write(&input);

                input.value = 0;
                handler.write(&input);

                input.value = 1;
                input.code = KEY_LEFTCTRL;
                handler.write(&input);
            }
        }

        // Pass-through
        handler.write(&input);
    }
    handler.ungrab();
}
