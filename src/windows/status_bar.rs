use crate::{
    Message,
    components::{
        canvas_background::{pill, ring},
        colored_bar,
        corner_curve::CornerCurve,
    },
    units,
    windows::shell_window::ShellWindow,
};
use chrono::Local;
use iced::{
    Alignment::Center,
    Color, Element, Font,
    Length::{Fill, Shrink},
    Point, Task,
    font::Weight,
    widget::{Button, button, canvas, image, row, space, svg, text},
};
use iced_exwlshell::reexport::{
    Anchor, BlurOption, KeyboardInteractivity, Layer, LayerSize, NewLayerShellSettings,
    OutputOption,
};
use lucide_icons::iced::icon_power;
use wb_dbus::sni::{NewTrayItem, TrayItemIcon, WatcherCommand};

pub struct StatusBar {
    pub monitor: String,
    time: String,
    tray_icons: Vec<NewTrayItem>,
    pub components: StatusBarComponents,
    pub is_main: bool,
}

bitflags::bitflags! {
    pub struct StatusBarComponents: u8 {
        const POWR = 0b0000_0001;
        const TIME = 0b0000_0010;
        const TRAY = 0b0000_0100;
    }
}

impl StatusBar {
    pub fn new() -> Self {
        let time = Local::now().format("%H:%M").to_string();
        StatusBar {
            monitor: "DP-3".to_string(),
            is_main: true,
            time,
            tray_icons: Vec::new(),
            components: StatusBarComponents::all(),
        }
    }
}

impl ShellWindow for StatusBar {
    fn layer_shell_settings(&self) -> NewLayerShellSettings {
        let horizontal_margin = if self.is_main {
            0
        } else {
            units::MARGIN as i32 * 2
        };
        let top_margin = units::MARGIN as i32;
        NewLayerShellSettings {
            size: LayerSize::fill_width(units::STATUS_BAR_HEIGHT as u32),
            layer: Layer::Top,
            anchor: Anchor::Left | Anchor::Top | Anchor::Right,
            exclusive_zone: Some(units::STATUS_BAR_HEIGHT as i32),
            margin: Some((top_margin, horizontal_margin, 0, horizontal_margin)),
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::OutputName(self.monitor.clone()),
            events_transparent: false,
            blur_option: BlurOption::None,
            namespace: Some("waybracelet".to_string()),
        }
    }

    fn update(&mut self, id: iced::window::Id, msg: &Message) -> Task<Message> {
        match msg {
            Message::Tick(_) => {
                self.time = Local::now().format("%H:%M").to_string();
                Task::none()
            }
            Message::TrayServer(cmd) if self.components.contains(StatusBarComponents::TRAY) => {
                match cmd {
                    WatcherCommand::ItemRegistered(new_tray_item) => {
                        self.tray_icons.push(new_tray_item.clone());
                        Task::none()
                    }
                    WatcherCommand::ItemUnregistered(_) => todo!(),
                }
            }
            Message::TrayIconClick(status_notifier_item_proxy) => {
                let status_notifier_item_proxy = status_notifier_item_proxy.clone();
                Task::perform(
                    async move {
                        status_notifier_item_proxy.activate(0, 0).await.unwrap();
                    },
                    |_| Message::Noop,
                )
            }
            _ => Task::none(),
        }
    }

    fn view(&self, _: iced::window::Id) -> Element<'_, Message> {
        let left_row = {
            let mut row = row![];

            let corner: Element<'_, Message> = if self.is_main {
                canvas(CornerCurve::new(
                    Point::new(
                        12.,
                        if self.is_main {
                            units::STATUS_BAR_HEIGHT
                        } else {
                            0.
                        },
                    ),
                    Point::new(units::STATUS_BAR_HEIGHT - 12., units::DESIGN_UNIT),
                    units::STROKE_WIDTH,
                ))
                .width(units::STATUS_BAR_HEIGHT - 12.)
                .height(units::STATUS_BAR_HEIGHT)
                .into()
            } else {
                ring(space().width(units::MARGIN).height(units::MARGIN)).into()
            };
            row = row.push(corner);
            row = row.push(colored_bar::horizontal_fill());

            row
        };
        let center_row = {
            row![
                colored_bar::horizontal_fill(),
                colored_bar::horizontal_fill(),
            ]
            .width(Fill)
        };
        let right_row = {
            let mut row = row![colored_bar::horizontal_fill()];

            if self.components.contains(StatusBarComponents::TRAY) && !self.tray_icons.is_empty() {
                let tray = tray_icons(&self.tray_icons);
                row = row.push(tray);
                row = row.push(colored_bar::h8());
            }

            if self.components.contains(StatusBarComponents::TIME) {
                let time = ring(
                    pill(
                        text(&self.time)
                            .color(mothscheme::swatches::TEXT_L20)
                            .center()
                            .align_y(Center)
                            .align_x(Center)
                            .height(Fill)
                            .width(Fill)
                            .line_height(1.)
                            .font(Font {
                                weight: Weight::Bold,
                                ..Font::with_name("Inter")
                            }),
                    )
                    .padding([0, 12]),
                )
                .width(Shrink);
                row = row.push(time);
                row = row.push(colored_bar::h8());
            }

            if self.components.contains(StatusBarComponents::POWR) {
                let powr = ring(off_button()).center(units::STATUS_BAR_HEIGHT);
                row = row.push(powr);
                row = row.push(colored_bar::h8());
            }

            let corner: Element<'_, Message> = if self.is_main {
                canvas(CornerCurve::new(
                    Point::new(units::DESIGN_UNIT, units::STATUS_BAR_HEIGHT),
                    Point::new(0., units::DESIGN_UNIT),
                    units::STROKE_WIDTH,
                ))
                .width(units::STATUS_BAR_HEIGHT - 12.)
                .height(units::STATUS_BAR_HEIGHT)
                .into()
            } else {
                ring(space().width(units::MARGIN).height(units::MARGIN)).into()
            };
            row = row.push(corner);

            row.width(Fill)
        };

        let row = row![
            left_row.height(Fill).align_y(Center),
            center_row.height(Fill).align_y(Center),
            right_row.height(Fill).align_y(Center),
        ];

        row.height(Fill).align_y(Center).into()
    }

    fn kind(&self) -> super::shell_window::Kind {
        super::shell_window::Kind::StatusBar
    }
}

fn off_button<'a>() -> Button<'a, Message> {
    button(icon_power().size(20).center())
        .on_press(Message::OpenLeaving)
        .style(|theme, _| button::Style {
            background: Some(iced::Background::Color(theme.palette().background)),
            text_color: theme.palette().text,
            border: iced::Border::default().rounded(units::PILL_BUTTON_RADIUS),
            shadow: iced::Shadow::default(),
            snap: true,
        })
        .height(Fill)
        .width(Fill)
}

fn tray_icons<'a>(trays: &'a [NewTrayItem]) -> Element<'a, Message> {
    ring(
        pill(
            row(trays.iter().map(tray_icon))
                .spacing(units::MARGIN)
                .padding([0., units::MARGIN])
                .width(Shrink),
        )
        .width(Shrink),
    )
    .into()
}

fn tray_icon<'a>(tray: &'a NewTrayItem) -> Element<'a, Message> {
    let icon_size = (units::STATUS_BAR_HEIGHT - units::STROKE_WIDTH * 4.) * 0.75;
    let icon: Element<'a, _> = match tray.icon {
        TrayItemIcon::IconData(width, height, ref pixels) => image(image::Handle::from_rgba(
            width as u32,
            height as u32,
            pixels.clone(),
        ))
        .height(icon_size)
        .width(icon_size)
        .into(),
        TrayItemIcon::IconName(ref name) => {
            let Some(icon) = freedesktop_icons::lookup(name)
                .with_size(32)
                .with_theme("Mothscheme")
                .force_svg()
                .find()
            else {
                todo!()
            };

            match icon.extension() {
                Some(ext) if ext == "svg" => svg(icon).height(icon_size).width(icon_size).into(),
                None | Some(_) => image(icon).height(icon_size).width(icon_size).into(),
            }
        }
        TrayItemIcon::NoIcon => todo!(),
    };
    button(icon)
        .on_press(Message::TrayIconClick(tray.proxy.clone()))
        .padding(0)
        .style(|theme, status| button::Style::default().with_background(Color::TRANSPARENT))
        .into()
}
