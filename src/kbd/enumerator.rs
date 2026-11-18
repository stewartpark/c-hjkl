use std::fs::File;
use std::io::Read;

#[derive(Debug, PartialEq)]
pub struct Keyboard {
    pub name: String,
    pub device_path: String,
}

#[derive(Debug, PartialEq)]
enum ProcBusInputDeviceEntryLine {
    EV(String),
    Name(String),
    Handlers(String),
    NotParsed,
}

pub fn enumerate_keyboards() -> std::io::Result<Vec<Keyboard>> {
    let mut file =
        File::open("/proc/bus/input/devices").expect("Unable to open /proc/bus/input/devices");
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(parse_proc_bus_input_devices(&contents))
}

fn parse_proc_bus_input_devices(contents: &str) -> Vec<Keyboard> {
    let entries = contents.split("\n\n").collect::<Vec<&str>>();
    let keyboards: Vec<Keyboard> = entries
        .into_iter()
        .map(|entry| {
            entry
                .split('\n')
                .map(|line| {
                    let parts = line.split('=').collect::<Vec<&str>>();
                    if line.starts_with("N: Name=") {
                        ProcBusInputDeviceEntryLine::Name(
                            parts[1].replace("\"", "").trim().to_string(),
                        )
                    } else if line.starts_with("B: EV=") {
                        ProcBusInputDeviceEntryLine::EV(
                            parts[1].replace("\"", "").trim().to_string(),
                        )
                    } else if line.starts_with("H: Handlers=") {
                        if let Some(t) = parts[1].replace("\"", "").trim().split(' ').last() {
                            ProcBusInputDeviceEntryLine::Handlers(t.to_string())
                        } else {
                            ProcBusInputDeviceEntryLine::NotParsed
                        }
                    } else {
                        ProcBusInputDeviceEntryLine::NotParsed
                    }
                })
                .filter(|x| *x != ProcBusInputDeviceEntryLine::NotParsed)
                .collect()
        })
        .map(|mut parsed_lines: Vec<ProcBusInputDeviceEntryLine>| {
            if parsed_lines.len() == 0 {
                None
            } else {
                let name = parsed_lines
                    .iter_mut()
                    .map(|line| {
                        if let ProcBusInputDeviceEntryLine::Name(s) = line {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .filter_map(|x| x)
                    .collect::<Vec<String>>()[0]
                    .clone();
                let device_path = parsed_lines
                    .iter_mut()
                    .map(|line| {
                        if let ProcBusInputDeviceEntryLine::Handlers(s) = line {
                            Some(format!("/dev/input/{}", s))
                        } else {
                            None
                        }
                    })
                    .filter_map(|x| x)
                    .collect::<Vec<String>>()[0]
                    .clone();
                let ev = parsed_lines
                    .iter_mut()
                    .map(|line| {
                        if let ProcBusInputDeviceEntryLine::EV(s) = line {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .filter_map(|x| x)
                    .collect::<Vec<String>>()[0]
                    .clone();

                // Only select fully functioning keyboards.
                if ev == "120013" || name.starts_with("Logitech ERGO") {
                    Some(Keyboard { name, device_path })
                } else {
                    None
                }
            }
        })
        .filter_map(|x| x)
        .collect::<Vec<Keyboard>>();
    keyboards
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic /proc/bus/input/devices entry.
    fn entry(name: &str, ev: &str, handlers: &str) -> String {
        format!(
            "I: Bus=0003 Vendor=0000 Product=0000 Version=0000\n\
             N: Name=\"{name}\"\n\
             P: Phys=phys0\n\
             S: Sysfs=/devices/input0\n\
             U: Uniq=\n\
             H: Handlers={handlers}\n\
             B: PROP=0\n\
             B: EV={ev}"
        )
    }

    #[test]
    fn real_world_fixture_yields_the_two_keyboards() {
        let keyboards = parse_proc_bus_input_devices(include_str!("proc_bus_input_devices.txt"));
        let expected = vec![
            Keyboard {
                name: "AT Translated Set 2 keyboard".to_string(),
                device_path: "/dev/input/event4".to_string(),
            },
            Keyboard {
                name: "Topre Corporation HHKB Professional".to_string(),
                device_path: "/dev/input/event5".to_string(),
            },
        ];
        assert_eq!(keyboards, expected);
    }

    #[test]
    fn empty_input_yields_no_keyboards() {
        assert_eq!(parse_proc_bus_input_devices(""), Vec::<Keyboard>::new());
    }

    #[test]
    fn blank_lines_only_yield_no_keyboards() {
        assert_eq!(
            parse_proc_bus_input_devices("\n\n\n"),
            Vec::<Keyboard>::new()
        );
    }

    #[test]
    fn keyboard_with_ev_120013_is_detected() {
        let keyboards = parse_proc_bus_input_devices(&entry(
            "Topre Corporation HHKB Professional",
            "120013",
            "sysrq kbd leds event5",
        ));
        assert_eq!(keyboards, vec![Keyboard {
            name: "Topre Corporation HHKB Professional".to_string(),
            device_path: "/dev/input/event5".to_string(),
        }]);
    }

    #[test]
    fn logitech_ergo_is_detected_by_name_even_without_keyboard_ev() {
        let keyboards = parse_proc_bus_input_devices(&entry("Logitech ERGO K870", "3", "kbd event9"));
        assert_eq!(keyboards, vec![Keyboard {
            name: "Logitech ERGO K870".to_string(),
            device_path: "/dev/input/event9".to_string(),
        }]);
    }

    #[test]
    fn non_keyboard_devices_are_ignored() {
        let contents = [
            entry("PixArt USB Optical Mouse", "17", "mouse0 event6"),
            entry("Lid Switch", "21", "event0"),
            entry("HDA Intel PCH Mic", "21", "event12"),
        ]
        .join("\n\n");
        assert_eq!(parse_proc_bus_input_devices(&contents), Vec::<Keyboard>::new());
    }

    #[test]
    fn device_path_uses_the_last_handler() {
        let keyboards = parse_proc_bus_input_devices(&entry(
            "AT Translated Set 2 keyboard",
            "120013",
            "sysrq kbd leds event4",
        ));
        assert_eq!(keyboards[0].device_path, "/dev/input/event4");
    }

    #[test]
    fn quotes_are_stripped_from_names() {
        let keyboards = parse_proc_bus_input_devices(&entry("Some \"Quoted\" Kbd", "120013", "event1"));
        assert_eq!(keyboards[0].name, "Some Quoted Kbd");
    }

    /// Characterization: an entry that parses lines but has no `N: Name=`
    /// (or no `H: Handlers=` / `B: EV=`) currently panics on an empty
    /// index. Pinned here so any future fix is a deliberate change.
    #[test]
    #[should_panic]
    fn malformed_entry_without_name_panics() {
        parse_proc_bus_input_devices("B: EV=120013");
    }
}
