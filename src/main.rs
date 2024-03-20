use iced::{Element, Sandbox, Settings, Theme};
use iced::widget::{button, column, container, row, text, text_editor};

fn main() -> iced::Result{
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
}

impl Sandbox for Editor {
    type Message = Message;

    fn new() -> Self {
        Self {
            content: text_editor::Content::with(include_str!("main.rs")),
        }
    }

    fn title(&self) -> String {
        String::from("TxtGrind")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let controls = row![button("Load File")];

        let position = {
            let (line, column) = self.content.cursor_position();

            text(format!("{}:{}", line, column))
        };

        let input = text_editor(&self.content).on_edit(Message::Edit);

        container(column![controls, input, position]).padding(20).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}