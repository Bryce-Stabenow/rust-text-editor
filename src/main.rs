use iced::{Element, Settings, Sandbox};
use iced::widget::text;

fn main() -> iced::Result{
    Editor::run(Settings::default())
}

struct Editor;

#[derive(Debug)]
enum Message {}

impl Sandbox for Editor {
    type Message = Message;

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("TxtGrind")
    }

    fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> Element<'_, Self::Message> {
        text("Welcome to TxtGrind").into()
    }
}