use crate::components::colored_bar::{horizontal_fill, vertical_fill};
use crate::components::corner_curve::CornerCurve;
use crate::windows::shell_window::ShellWindow;
use crate::{Message, units};
use iced::Alignment::Center;
use iced::widget::row;
use iced::{Element, Length::Fill, Task, widget::column};
use iced_layershell::reexport::{
    Anchor, BlurOption, KeyboardInteractivity, Layer, LayerSize, NewLayerShellSettings,
    OutputOption,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Left,
    Right,
    Bottom,
}

#[derive(Clone, Copy)]
pub struct EdgeBar {
    position: Position,
}

impl EdgeBar {
    pub fn new(position: Position) -> Self {
        EdgeBar { position }
    }
}

impl ShellWindow for EdgeBar {
    fn layer_shell_settings(&self) -> NewLayerShellSettings {
        let (size, anchor) = match self.position {
            Position::Left => (
                LayerSize::fill_height(units::DESIGN_UNIT as u32),
                Anchor::Left | Anchor::Top | Anchor::Bottom,
            ),
            Position::Right => (
                LayerSize::fill_height(units::DESIGN_UNIT as u32),
                Anchor::Right | Anchor::Top | Anchor::Bottom,
            ),
            Position::Bottom => (
                LayerSize::fill_width(units::DESIGN_UNIT as u32),
                Anchor::Left | Anchor::Right | Anchor::Bottom,
            ),
        };

        NewLayerShellSettings {
            size,
            layer: Layer::Top,
            anchor,
            exclusive_zone: Some(units::DESIGN_UNIT as i32),
            margin: None,
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::OutputName("DP-3".to_string()),
            events_transparent: true,
            blur_option: BlurOption::None,
            namespace: Some("waybracelet".to_string()),
        }
    }

    fn update(&mut self, _: iced::window::Id, _: &Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self, _: iced::window::Id) -> Element<'_, Message> {
        match self.position {
            Position::Bottom => row![horizontal_fill(units::STROKE_WIDTH)]
                .height(Fill)
                .align_y(Center)
                .into(),
            Position::Left | Position::Right => column![
                vertical_fill(units::STROKE_WIDTH),
                match self.position {
                    Position::Left =>
                        CornerCurve::top_right(units::DESIGN_UNIT, units::STROKE_WIDTH),
                    Position::Right =>
                        CornerCurve::top_left(units::DESIGN_UNIT, units::STROKE_WIDTH),
                    _ => unreachable!(),
                },
            ]
            .height(Fill)
            .align_x(Center)
            .into(),
        }
    }

    fn kind(&self) -> super::shell_window::Kind {
        super::shell_window::Kind::EdgeBar
    }
}
