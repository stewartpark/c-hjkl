pub fn run_forever_until_keyboards_change() {
    let mut count: i8 = -1;
    loop {
        if let Ok(keyboards) = crate::kbd::enumerator::enumerate_keyboards() {
            if count == -1 { count = keyboards.len() as i8; }
            if count != keyboards.len() as i8 { break; }
        }
    }
}
