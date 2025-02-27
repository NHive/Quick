use enigo::{
    Direction::{Click, Press, Release},
    Enigo, InputError, Key, Keyboard, Settings,
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
    thread::sleep(Duration::from_millis(100));

    enigo.key(Key::Shift, Press)?;
    enigo.key(Key::Other(0x2D), Click)?;
    enigo.key(Key::Shift, Release)?;
    Ok(())
}
