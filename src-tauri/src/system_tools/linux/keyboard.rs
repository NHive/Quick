use enigo::{
    Direction::{Click, Press, Release},
    Enigo, InputError, Key, Keyboard, NewConError, Settings,
};
use std::thread;
use std::time::Duration;

#[allow(unused)]
pub fn copy() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.key(Key::Control, Press);
    enigo.key(Key::Unicode('c'), Click);
    enigo.key(Key::Control, Release);
}

pub fn paste() -> Result<(), InputError> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.key(Key::Control, Press)?;
    thread::sleep(Duration::from_millis(10));
    enigo.key(Key::Unicode('v'), Click)?;
    thread::sleep(Duration::from_millis(10));
    enigo.key(Key::Control, Release)?;
    Ok(())
}
