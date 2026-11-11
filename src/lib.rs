pub mod data;
pub mod font;
pub mod screen;
pub mod style;
pub mod jafl;

/// An iced Element parameterized with our custom Theme.
pub type Element<'a, Message> = iced::Element<'a, Message, data::Theme>;
