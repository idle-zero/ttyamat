use iced::keyboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    OpenTab,
    CloseActiveTab,
    SelectNextTab,
    SelectPreviousTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resolution {
    Execute(Action),
    Suppress,
    Unhandled,
}

/// Resolves application bindings before the event is encoded as terminal input.
pub(crate) fn resolve(event: &keyboard::Event) -> Resolution {
    match binding(event) {
        Some(_) if is_repeated_keypress(event) => Resolution::Suppress,
        Some(action) => Resolution::Execute(action),
        None => Resolution::Unhandled,
    }
}

fn binding(event: &iced::keyboard::Event) -> Option<Action> {
    let keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        ..
    } = event
    else {
        return None;
    };
    let command_shift = keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT;
    let latin_key = key
        .to_latin(*physical_key)
        .map(|character| character.to_ascii_lowercase());
    if *modifiers == command_shift {
        match latin_key {
            Some('t') => return Some(Action::OpenTab),
            Some('w') => return Some(Action::CloseActiveTab),
            _ => {}
        }
    }

    if !matches!(key, keyboard::Key::Named(keyboard::key::Named::Tab)) {
        return None;
    }

    if *modifiers == keyboard::Modifiers::CTRL {
        Some(Action::SelectNextTab)
    } else if *modifiers == (keyboard::Modifiers::CTRL | keyboard::Modifiers::SHIFT) {
        Some(Action::SelectPreviousTab)
    } else {
        None
    }
}

fn is_repeated_keypress(event: &keyboard::Event) -> bool {
    matches!(event, keyboard::Event::KeyPressed { repeat: true, .. })
}
