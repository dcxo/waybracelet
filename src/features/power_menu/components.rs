use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    border::rounded,
    widget::{Text, button},
};

use crate::{Message, features::power_menu::PowerMenuMessage};

pub fn power_button<'a>(
    icon: Text<'a>,
    on_press: PowerMenuMessage,
) -> impl Into<Element<'a, Message>> {
    button(
        icon.size(56)
            .height(Fill)
            .width(Fill)
            .align_x(Center)
            .align_y(Center),
    )
    .style(|theme, status| {
        let mut style = button::primary(theme, status);
        style.text_color = theme.palette().text;
        style.border = rounded(i32::MAX);
        style.with_background(theme.palette().background)
    })
    .on_press(Message::PowerMenu(on_press))
    .width(56 * 2)
    .height(56 * 2)
}
