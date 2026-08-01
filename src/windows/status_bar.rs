use crate::components::canvas_background::{pill, ring};
use crate::components::colored_bar;
use crate::components::corner_curve::CornerCurve;
use crate::windows::shell_window::ShellWindow;
use crate::{Message, units};
use chrono::Local;
use iced::Color;
use iced::Length::Shrink;
use iced::time::Instant;
use iced::widget::image::Handle;
use iced::widget::{Button, Row, button, image, space, svg};
use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Point, Task,
    widget::{canvas, row, text},
};
use iced_layershell::reexport::{
    Anchor, BlurOption, KeyboardInteractivity, Layer, LayerSize, NewLayerShellSettings,
    OutputOption,
};
use lucide_icons::iced::icon_power;
use wb_dbus::sni::{NewTrayItem, StatusNotifierItemProxy, TrayItemIcon, WatcherCommand};

pub struct StatusBar {
    time: String,
    tray_icons: Vec<NewTrayItem>,
}

impl StatusBar {
    pub fn new() -> Self {
        let time = Local::now().format("%H:%M").to_string();
        StatusBar {
            time,
            tray_icons: Vec::new(),
        }
    }
}

impl ShellWindow for StatusBar {
    fn layer_shell_settings(&self) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: LayerSize::fill_width(units::STATUS_BAR_HEIGHT as u32),
            layer: Layer::Top,
            anchor: Anchor::Left | Anchor::Top | Anchor::Right,
            exclusive_zone: Some(units::STATUS_BAR_HEIGHT as i32),
            margin: Some((units::MARGIN as i32, 0, 0, 0)),
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::OutputName("DP-3".to_string()),
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
            Message::TrayServer(cmd) => match cmd {
                WatcherCommand::ItemRegistered(new_tray_item) => {
                    self.tray_icons.push(new_tray_item.clone());
                    Task::none()
                }
                WatcherCommand::ItemUnregistered(_) => todo!(),
            },
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
        let v = ring(
            pill(
                text(&self.time)
                    .color(mothscheme::swatches::TEXT_L20)
                    .center()
                    .align_y(Center)
                    .align_x(Center)
                    .height(Fill)
                    .width(Fill)
                    .line_height(1.)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..iced::Font::DEFAULT
                    }),
            )
            .padding([0, 12]),
        )
        .height(units::STATUS_BAR_HEIGHT)
        .width(Shrink)
        .align_y(Center)
        .align_x(Center);

        row![
            canvas(CornerCurve::new(
                Point::new(12., units::STATUS_BAR_HEIGHT),
                Point::new(units::STATUS_BAR_HEIGHT - 12., units::DESIGN_UNIT),
                units::STROKE_WIDTH
            ))
            .width(units::STATUS_BAR_HEIGHT - 12.)
            .height(units::STATUS_BAR_HEIGHT),
            colored_bar::horizontal_fill(units::STROKE_WIDTH),
            tray_icons(&self.tray_icons),
            colored_bar::h8(),
            v,
            colored_bar::h8(),
            ring(off_button()).center(units::STATUS_BAR_HEIGHT),
            colored_bar::h24(),
            canvas(CornerCurve::new(
                Point::new(units::DESIGN_UNIT, units::STATUS_BAR_HEIGHT),
                Point::new(0., units::DESIGN_UNIT),
                units::STROKE_WIDTH
            ))
            .width(units::STATUS_BAR_HEIGHT - 12.)
            .height(units::STATUS_BAR_HEIGHT),
        ]
        .height(Fill)
        .align_y(Center)
        .into()
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
    if trays.is_empty() {
        return space().into();
    }
    ring(
        pill(
            row(trays.iter().map(tray_icon))
                .spacing(units::STROKE_WIDTH)
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
        TrayItemIcon::IconData(width, height, ref pixels) => image(Handle::from_rgba(
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
