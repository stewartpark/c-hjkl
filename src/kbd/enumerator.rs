use std::fs::File;
use std::io::Read;

#[derive(Debug)]
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

fn parse_proc_bus_input_devices(contents: &String) -> Vec<Keyboard> {
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

    #[test]
    fn it_works() {
        let test_txt = include_str!("proc_bus_input_devices.txt").to_string();
        let keyboards = parse_proc_bus_input_devices(&test_txt);
        assert_eq!(keyboards.len(), 2);
    }
}
