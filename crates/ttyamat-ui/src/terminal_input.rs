use iced::keyboard::{self, Key, key::Named};

const ESCAPE: u8 = 0x1b;

/// Encodes the basic keyboard input currently supported by the terminal UI.
///
/// Application shortcuts must be handled before calling this function.
pub(crate) fn encode(event: &keyboard::Event) -> Option<Vec<u8>> {
    let keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        text,
        ..
    } = event
    else {
        return None;
    };

    // TODO: Alt/AltGr and Super/Command behavior will be added with the extended
    // keyboard protocol support. Do not emit misleading bytes in the meantime.
    if modifiers.alt() || modifiers.logo() {
        return None;
    }

    if modifiers.control() {
        let character = key.to_latin(*physical_key)?.to_ascii_lowercase();

        return character
            .is_ascii_lowercase()
            .then_some(vec![(character as u8) & 0x1f]);
    }

    if let Key::Named(named) = key {
        return encode_named(*named);
    }

    text.as_ref()
        .map(AsRef::as_ref)
        .filter(|text: &&str| !text.is_empty())
        .map(|text| text.as_bytes().to_vec())
}

fn encode_named(key: Named) -> Option<Vec<u8>> {
    match key {
        Named::Enter => Some(vec![b'\r']),
        Named::Backspace => Some(vec![0x7f]),
        Named::Tab => Some(vec![b'\t']),
        Named::Space => Some(vec![b' ']),
        Named::Escape => Some(vec![ESCAPE]),
        Named::ArrowUp => Some(b"\x1b[A".to_vec()),
        Named::ArrowDown => Some(b"\x1b[B".to_vec()),
        Named::ArrowRight => Some(b"\x1b[C".to_vec()),
        Named::ArrowLeft => Some(b"\x1b[D".to_vec()),
        _ => None,
    }
}
