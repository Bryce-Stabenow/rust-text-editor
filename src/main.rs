use std::io::{self};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::widget::{button, column, container, horizontal_space, row, text, text_editor};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};

fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
}

impl Application for Editor {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                path: None,
                content: text_editor::Content::with(include_str!("main.rs")),
                error: None,
            },
            Command::perform(load_file(default_file()), Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("TxtGrind")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
                Command::none()
            }
            Message::Open => Command::perform(pick_file(), Message::FileOpened),
            Message::FileOpened(Ok((path, result))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&result);
                Command::none()
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let controls = row![button("Load File").on_press(Message::Open)];

        let position = {
            let (line, column) = self.content.cursor_position();

            text(format!("{}:{}", line, column))
        };

        let file_path = match self.path.as_deref().and_then(Path::to_str) {
            Some(path) => text(path).size(14),
            _ => text(""),
        };

        let status_bar = row![file_path, horizontal_space(Length::Fill), position];

        let input = text_editor(&self.content).on_edit(Message::Edit);

        container(column![controls, input, status_bar])
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IO)?;

    Ok((path, contents))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(handle.path().to_owned()).await
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind),
}
