use std::time::Duration;

use iced::{
    Border, Element,
    Length::Fill,
    Shadow, Task,
    border::Radius,
    widget::{Button, button, container, grid, mouse_area},
};
use iced_exwlshell::reexport::{
    Anchor, BlurOption, KeyboardInteractivity, LayerSize, NewLayerShellSettings,
};
use lucide_icons::iced::{
    icon_lock, icon_log_out, icon_moon, icon_power, icon_rotate_ccw, icon_snowflake,
};
use smol::Timer;

use crate::{Message, WeLeavingMessage, units, windows::shell_window::ShellWindow};

#[derive(Default)]
pub struct WeLeaving {
    confirmed: bool,
}

impl ShellWindow for WeLeaving {
    fn layer_shell_settings(&self) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: LayerSize::FILL,
            layer: iced_exwlshell::reexport::Layer::Overlay,
            anchor: Anchor::all(),
            exclusive_zone: Some(0),
            margin: Some((
                -(units::STATUS_BAR_HEIGHT as i32),
                -(units::DESIGN_UNIT as i32),
                -(units::DESIGN_UNIT as i32),
                -(units::DESIGN_UNIT as i32),
            )),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            output_option: iced_exwlshell::reexport::OutputOption::Active,
            events_transparent: false,
            blur_option: BlurOption::None,
            namespace: Some("Leave".to_string()),
        }
    }

    fn update(&mut self, id: iced::window::Id, msg: &Message) -> iced::Task<Message> {
        match msg {
            Message::LeaveAction(msg) => {
                match msg {
                    WeLeavingMessage::Off => {
                        if self.confirmed {
                            let _ = system_shutdown::shutdown();
                        } else {
                            self.confirmed = true;
                            return Task::perform(
                                async { Timer::after(Duration::from_secs(5)).await },
                                |_| Message::Deconfirm,
                            );
                        }
                    }
                    WeLeavingMessage::Reboot => {
                        let _ = system_shutdown::reboot();
                    }
                    WeLeavingMessage::LogOut => {
                        let _ = system_shutdown::logout();
                    }
                    WeLeavingMessage::Suspend => {
                        let _ = system_shutdown::sleep();
                    }
                    WeLeavingMessage::LockOff => {}
                    WeLeavingMessage::Hibernate => {
                        let _ = system_shutdown::hibernate();
                    }
                };
            }
            Message::Deconfirm => {
                self.confirmed = false;
            }
            Message::EscapePressed => {
                return Task::done(Message::Close(id));
            }
            _ => (),
        };

        Task::none()
    }

    fn view(&self, id: iced::window::Id) -> iced::Element<'_, Message> {
        let buttons = [
            (
                icon_log_out(),
                mothscheme::swatches::ORANGE_L40,
                WeLeavingMessage::LogOut,
            ),
            (
                icon_lock(),
                mothscheme::swatches::YELLOW_L40,
                WeLeavingMessage::LockOff,
            ),
            (
                icon_moon(),
                mothscheme::swatches::PINK_L40,
                WeLeavingMessage::Suspend,
            ),
            (
                icon_snowflake(),
                mothscheme::swatches::CYAN_L40,
                WeLeavingMessage::Hibernate,
            ),
            (
                icon_power(),
                mothscheme::swatches::RED_L40,
                WeLeavingMessage::Off,
            ),
            (
                icon_rotate_ccw(),
                mothscheme::swatches::GREEN_L40,
                WeLeavingMessage::Reboot,
            ),
        ]
        .into_iter()
        .enumerate()
        .map(|(idx, (icon, color, message))| {
            leaving_button(
                icon.size(64)
                    .color(if self.confirmed && message == WeLeavingMessage::Off {
                        mothscheme::BOMBYX.text
                    } else {
                        color
                    }),
                self.confirmed,
                Message::LeaveAction(message),
                idx as u8,
            )
            .into()
        });

        mouse_area(
            container(
                container(
                    grid(buttons)
                        .height(Fill)
                        .columns(3)
                        .spacing(units::STROKE_WIDTH),
                )
                .padding(units::MARGIN)
                .height(units::SCREEN_HEIGHT as f32 / 3.)
                .width(units::SCREEN_WIDTH as f32 / 3.)
                .style(|theme| container::Style {
                    text_color: Some(theme.palette().text),
                    background: None,
                    border: iced::Border::default()
                        .color(theme.palette().background)
                        .width(units::STROKE_WIDTH)
                        .rounded(Radius::new(units::SCREEN_HEIGHT as f32 / 3. / 5.)),
                    shadow: Shadow::default(),
                    snap: false,
                })
                .clip(true),
            )
            .center(Fill)
            .style(|theme| {
                container::Style::default()
                    .color(theme.palette().text)
                    .background(theme.palette().text.scale_alpha(0.33))
            }),
        )
        .on_press(Message::RemoveWindow(id))
        .into()
    }

    fn kind(&self) -> super::shell_window::Kind {
        super::shell_window::Kind::Leaving
    }
}

fn leaving_button<'a>(
    content: impl Into<Element<'a, Message>>,
    confirmed: bool,
    message: Message,
    idx: u8,
) -> Button<'a, Message> {
    let radius = match idx {
        0 => Radius::default().top_left((units::SCREEN_HEIGHT as f32 / 3. / 5.) - units::MARGIN),
        2 => Radius::default().top_right((units::SCREEN_HEIGHT as f32 / 3. / 5.) - units::MARGIN),
        3 => Radius::default().bottom_left((units::SCREEN_HEIGHT as f32 / 3. / 5.) - units::MARGIN),
        5 => {
            Radius::default().bottom_right((units::SCREEN_HEIGHT as f32 / 3. / 5.) - units::MARGIN)
        }
        _ => Radius::default(),
    };
    let is_off = matches!(message, Message::LeaveAction(WeLeavingMessage::Off));
    button(container(content).center(Fill))
        .style({
            move |theme, status| {
                let style = button::Style {
                    background: None,
                    text_color: theme.palette().text,
                    border: Border::default()
                        .rounded(radius)
                        .color(theme.palette().background),
                    shadow: Shadow::default(),
                    snap: true,
                };

                style.with_background(match status {
                    button::Status::Active if confirmed && is_off => mothscheme::swatches::RED_L40,
                    button::Status::Active => theme.palette().background,
                    button::Status::Hovered if confirmed && is_off => mothscheme::swatches::RED_L60,
                    button::Status::Hovered => mothscheme::swatches::BACKGROUND_L60,
                    button::Status::Pressed if confirmed && is_off => mothscheme::swatches::RED_L70,
                    button::Status::Pressed => mothscheme::swatches::BACKGROUND_L70,
                    button::Status::Disabled => mothscheme::swatches::BACKGROUNDW_L25,
                })
            }
        })
        .on_press(message)
}
