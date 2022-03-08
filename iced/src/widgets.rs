use iced::text_input::{State, TextInput};

pub fn text_input(state: &mut State, prompt: &str, default: Option<Self>) -> TextInput<Message> {
    TextInput::new(
        &mut state,
        prompt,
        self.value
            .as_ref()
            .map(u32::to_string)
            .as_ref()
            .map(String::as_str)
            .unwrap_or(""),
        Event::InputChanged,
    )
}
