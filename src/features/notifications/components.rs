use std::time::Instant;

use iced::{
    Element,
    Length::{Fill, Shrink},
    Padding, Shadow, Vector,
    border::{Radius, rounded},
    time::milliseconds,
    widget::{Row, button, column, container, float, sensor, value},
};

use crate::{
    Message,
    features::notifications::{AnimatedNotification, Notifications, NotificationsMessage},
};

pub fn notification<'a>(
    now: Instant,
    notification: &'a AnimatedNotification,
) -> Element<'a, Message> {
    let n_notification = notification.notification.clone();

    let actions = Row::from_iter(n_notification.actions.iter().map(|(key, text)| {
        dbg!(&key, &text);
        button(value(text).width(Fill).center()).into()
    }));

    sensor(
        float(
            container(column![
                value(n_notification.app_name),
                value(n_notification.summary),
                value(n_notification.body),
                actions
            ])
            .padding(Padding::new(
                (Notifications::ROUND - Notifications::PAD) * 2. / 3.,
            ))
            .style(|theme: &iced::Theme| container::Style {
                text_color: Some(theme.palette().text),
                background: Some(theme.palette().background.into()),
                border: rounded(Radius::default().left(Notifications::ROUND - Notifications::PAD)),
                shadow: Shadow::default(),
                snap: true,
            })
            .height(Shrink)
            .width(Fill),
        )
        .translate(move |_, _| {
            Vector::new(
                notification
                    .animation
                    .interpolate_with(|f| f * 500., now + milliseconds(100)),
                0.,
            )
        }),
    )
    .key(n_notification.id)
    .on_show(move |s| {
        dbg!(s);
        Message::Notifications(NotificationsMessage::PopUp(n_notification.id))
    })
    .into()
}
