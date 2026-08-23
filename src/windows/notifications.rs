use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Duration;

use iced::Border;
use iced::animation::{Animation, Easing};
use iced::time::Instant;
use iced::widget::container as iced_container;
use iced::{Background, Function};
use iced::{
    Element,
    Length::{Fill, Shrink},
    Shadow, Size, Task,
    alignment::Vertical::Bottom,
    widget::{button, column, float, keyed, mouse_area, row, sensor, space, text},
};
use iced_exwlshell::actions::ActionCallback;
use iced_exwlshell::reexport::{Anchor, BlurOption, KeyboardInteractivity, LayerSize};
use lucide_icons::iced::icon_x;
use wb_dbus::SignalEmitter;
use wb_dbus::notifications::{Notification, NotificationServerSignals};

use crate::Message;
use crate::{
    components::canvas_background::{pill_background_style, ring},
    units,
    windows::shell_window::ShellWindow,
};

const LIL_DELAY: Duration = Duration::from_millis(200);

struct NotificationAnim {
    slide: Animation<f32>,
    timeout: Option<Animation<bool>>,
    remaining: Option<Duration>,
}

pub struct Notifications {
    pub notifications: BTreeMap<u32, (Notification, NotificationAnim)>,
    last_tick: Instant,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            notifications: BTreeMap::new(),
            last_tick: Instant::now(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    fn start_close_animation(&mut self, id: u32, now: Instant) {
        let Some((_, anim)) = self.notifications.get_mut(&id) else {
            return;
        };

        anim.slide.go_mut(0.0, now);
    }

    fn remove_notification(&mut self, id: u32) -> Task<Message> {
        self.notifications.remove(&id);

        let is_empty = self.notifications.is_empty();

        Task::perform(
            async move {
                let Some(conn) = wb_dbus::connection() else {
                    return;
                };
                let signal_emitter =
                    SignalEmitter::new(conn, wb_dbus::notifications::PATH).unwrap();
                signal_emitter.notification_closed(id, 1).await.ok();
            },
            move |_| {
                if is_empty {
                    Message::CloseByKind(super::shell_window::Kind::Notification)
                } else {
                    Message::Noop
                }
            },
        )
    }
}

impl ShellWindow for Notifications {
    fn layer_shell_settings(&self) -> iced_exwlshell::reexport::NewLayerShellSettings {
        iced_exwlshell::reexport::NewLayerShellSettings {
            size: LayerSize::fill_height(units::NOTIFICATION_WIDTH as u32),
            layer: iced_exwlshell::reexport::Layer::Top,
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Right,
            exclusive_zone: None,
            margin: Some((
                units::MARGIN as i32,
                units::MARGIN as i32,
                units::MARGIN as i32,
                units::MARGIN as i32,
            )),
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: iced_exwlshell::reexport::OutputOption::OutputName("DP-3".to_string()),
            events_transparent: true,
            blur_option: BlurOption::None,
            namespace: Some("Notifications".to_string()),
        }
    }

    fn is_animating(&self) -> bool {
        self.notifications.values().any(|a| {
            a.1.slide.is_animating(self.last_tick)
                || a.1
                    .timeout
                    .as_ref()
                    .is_some_and(|w| w.is_animating(self.last_tick))
        })
    }

    fn update(&mut self, id: iced::window::Id, msg: &Message) -> iced::Task<Message> {
        match msg {
            Message::Tick(now) => {
                self.last_tick = *now;

                let mut to_remove = Vec::with_capacity(self.notifications.len());
                for (id, (_, anim)) in self.notifications.iter_mut() {
                    if anim
                        .timeout
                        .as_ref()
                        .is_some_and(|timeout| !timeout.is_animating(self.last_tick))
                        && anim.remaining.is_none()
                    {
                        {
                            let now = self.last_tick;

                            anim.slide.go_mut(0.0, now);
                        };
                    }
                    if !anim.slide.is_animating(self.last_tick) && anim.slide.value() == 0.0 {
                        to_remove.push(*id);
                    }
                }

                Task::batch(to_remove.iter().map(|id| self.remove_notification(*id)))
            }
            Message::NotificationServer(cmd) => match cmd {
                wb_dbus::notifications::NotificationsCommand::NewNotification(notification) => {
                    self.last_tick = Instant::now();
                    let id = notification.id;
                    let timeout = notification.timeout_secs;

                    self.notifications.insert(
                        id,
                        (
                            notification.clone(),
                            NotificationAnim {
                                slide: Animation::new(0.0)
                                    .slow()
                                    .easing(Easing::EaseInOut)
                                    .go(1.0, self.last_tick),
                                timeout: timeout.map(|d| {
                                    Animation::new(false)
                                        .duration(d)
                                        .delay(LIL_DELAY)
                                        .go(true, self.last_tick)
                                }),
                                remaining: None,
                            },
                        ),
                    );

                    Task::none()
                }
                wb_dbus::notifications::NotificationsCommand::CloseNotification(id) => {
                    self.start_close_animation(*id, self.last_tick);
                    Task::none()
                }
                wb_dbus::notifications::NotificationsCommand::ReplaceNotification(
                    id,
                    notification,
                ) => {
                    self.last_tick = Instant::now();
                    let timeout = notification.timeout_secs;

                    self.notifications.insert(
                        *id,
                        (
                            notification.clone(),
                            NotificationAnim {
                                slide: Animation::new(1.0).slow().easing(Easing::EaseInOut),
                                timeout: timeout.map(|d| {
                                    Animation::new(false)
                                        .duration(d)
                                        .delay(LIL_DELAY)
                                        .go(true, self.last_tick)
                                }),
                                remaining: None,
                            },
                        ),
                    );

                    Task::none()
                }
            },
            Message::CloseNotification(id) => {
                self.start_close_animation(*id, self.last_tick);
                Task::none()
            }
            Message::HoverNotification(id) => {
                let Some((_, anim)) = self.notifications.get_mut(&id) else {
                    return Task::none();
                };
                anim.remaining = anim.timeout.as_ref().map(|t| t.remaining(self.last_tick));
                Task::none()
            }
            Message::UnhoverNotification(id) => {
                let Some((_, anim)) = self.notifications.get_mut(&id) else {
                    return Task::none();
                };
                anim.timeout = anim
                    .remaining
                    .take()
                    .map(|d| Animation::new(false).duration(d).go(true, self.last_tick));
                Task::none()
            }
            Message::OnActionActivated(id, action) => {
                let action = action.clone();
                let id = *id;
                Task::perform(
                    async move {
                        let Some(conn) = wb_dbus::connection() else {
                            return;
                        };
                        let signal_emitter =
                            SignalEmitter::new(conn, wb_dbus::notifications::PATH).unwrap();
                        signal_emitter.action_invoked(id, &action).await.ok();
                    },
                    |_| Message::Noop,
                )
            }
            Message::OnResize(rid, size) if *rid == id => {
                let size = *size;
                Task::done(Message::SetInputRegion {
                    id,
                    callback: ActionCallback::new(move |r| {
                        r.add(0, 0, size.width as i32, size.height as i32);
                    }),
                })
            }
            _ => Task::none(),
        }
    }

    fn view(&self, id: iced::window::Id) -> iced::Element<'_, Message> {
        let max_alpha = self
            .notifications
            .iter()
            .map(|(_, (_, anim))| anim.slide.value())
            .fold(0., f32::max);

        let kc = keyed::Column::new()
            .spacing(units::STROKE_WIDTH)
            .width(Fill)
            .height(Fill);

        let now = self.last_tick;
        let children = self
            .notifications
            .iter()
            .rev()
            .map(|(id, (notification, anim))| {
                let t = anim.slide.interpolate_with(|v| v, now);
                let alpha = t;
                let offset_x = (1.0 - t) * units::NOTIFICATION_WIDTH;
                (*id, notification_view(notification, alpha, offset_x))
            });

        let kc = kc.extend(children);

        sensor(ring(kc).width(Fill).height(Shrink))
            .on_resize(Message::OnResize.with(id))
            .into()
    }

    fn kind(&self) -> super::shell_window::Kind {
        super::shell_window::Kind::Notification
    }
}

fn notification_view<'a>(
    notification: &'a Notification,
    alpha: f32,
    offset_x: f32,
) -> Element<'a, Message> {
    let id = notification.id;

    let children = notification.actions.iter().flat_map(|actions| {
        actions.rest.iter().map(|(ida, loc)| {
            button(text(loc))
                .width(Fill)
                .on_press(Message::OnActionActivated(id, ida.clone()))
                .into()
        })
    });

    let mut content = mouse_area(
        iced_container(
            column![
                text(&notification.app_name)
                    .size(12)
                    .color(mothscheme::themes::styx::MUTED),
                row![
                    text(&notification.summary).font(iced::Font {
                        weight: iced::font::Weight::Black,
                        ..Default::default()
                    }),
                    space().width(Fill),
                    button(icon_x())
                        .style(|theme, _| button::Style {
                            background: None,
                            text_color: theme.palette().danger,
                            border: Border::default(),
                            shadow: Shadow::default(),
                            snap: false
                        })
                        .padding(0)
                        .on_press(Message::CloseNotification(id))
                ]
                .align_y(Bottom),
                text(&notification.body),
                row(children).spacing(units::STROKE_WIDTH)
            ]
            .width(Fill),
        )
        .center(Fill)
        .padding(units::MARGIN)
        .style(move |theme| {
            let mut style = pill_background_style(theme);
            if let Some(Background::Color(ref mut color)) = style.background {
                color.a = alpha;
            }
            style
        }),
    )
    .on_enter(Message::HoverNotification(id))
    .on_exit(Message::UnhoverNotification(id));

    if notification
        .actions
        .as_ref()
        .is_some_and(|actions| actions.default.is_some())
    {
        content = content.on_press(Message::OnActionActivated(id, "default".into()))
    }

    float(content)
        .translate(move |_, _| iced::Vector::new(offset_x, 0.0))
        .into()
}
