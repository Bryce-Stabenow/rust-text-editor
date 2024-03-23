use std::io::{self};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::highlighter::{self, Highlighter};
use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text, text_editor, tooltip,
};
use iced::{executor, theme, Application, Command, Element, Font, Length, Settings, Theme};

fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../fonts/editor-icons.ttf")
            .as_slice()
            .into()],
        ..Settings::default()
    })
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
    theme: highlighter::Theme,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    New,
    Open,
    Save,
    FileSaved(Result<PathBuf, Error>),
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    ThemeSelected(highlighter::Theme),
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
                content: text_editor::Content::new(),
                error: None,
                theme: highlighter::Theme::SolarizedDark,
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
                self.error = None;
                Command::none()
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                Command::none()
            }
            Message::Save => {
                let text = self.content.text();
                Command::perform(save_file(self.path.clone(), text), Message::FileSaved)
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
            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                Command::none()
            }
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let controls = row![
            tooltip(
                button(container(open_icon()).width(50).center_x()).on_press(Message::Open),
                "Open File",
                tooltip::Position::FollowCursor
            )
            .style(theme::Container::Box),
            tooltip(
                button(container(save_icon()).width(50).center_x()).on_press(Message::Save),
                "Save File",
                tooltip::Position::FollowCursor
            )
            .style(theme::Container::Box),
            tooltip(
                button(container(new_icon()).width(50).center_x()).on_press(Message::New),
                "New File",
                tooltip::Position::FollowCursor
            )
            .style(theme::Container::Box),
            horizontal_space(Length::Fill),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.theme),
                Message::ThemeSelected
            )
        ]
        .spacing(10)
        .padding([5, 0]);

        let status_bar = {
            let position = {
                let (line, column) = self.content.cursor_position();

                text(format!("{}:{}", line, column))
            };

            let status = if let Some(Error::IO(error)) = self.error {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(14),
                    None => text("New file"),
                }
            };

            row![status, horizontal_space(Length::Fill), position].padding([5, 0])
        };

        let input = text_editor(&self.content)
            .on_edit(Message::Edit)
            .highlight::<Highlighter>(
                {
                    highlighter::Settings {
                        theme: self.theme, // Theme we chose for the highlighting
                        extension: self
                            .path
                            .as_ref()
                            .and_then(|path| path.extension()?.to_str())
                            .unwrap_or("rs")
                            .to_string(), // This is the file extension we want to use to tell the highlighter what language the file is in
                    }
                },
                |highlight, _theme| highlight.to_format(),
            );

        container(column![controls, input, status_bar])
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        match self.theme.is_dark() {
            true => Theme::Dark,
            false => Theme::Light,
        }
    }
}

fn new_icon<'a>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn save_icon<'a>() -> Element<'a, Message> {
    icon('\u{E801}')
}

fn open_icon<'a>() -> Element<'a, Message> {
    icon('\u{F115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
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

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, text)
        .await
        .map_err(|err| Error::IO(err.kind()))?;

    Ok(path)
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind),
}
