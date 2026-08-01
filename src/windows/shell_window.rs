use iced::Element;
use iced::Task;
use iced::window;
use iced_layershell::reexport::NewLayerShellSettings;

use crate::Message;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Kind {
    EdgeBar,
    Leaving,
    Notification,
    SpotLight,
    StatusBar,
}

pub struct Window {
    pub id: window::Id,
    inner: Box<dyn ShellWindow>,
}

impl Window {
    pub fn new(inner: impl ShellWindow + 'static) -> Self {
        Self {
            id: window::Id::unique(),
            inner: Box::new(inner),
        }
    }
}

pub trait ShellWindow {
    fn open(self) -> (Window, Task<Message>)
    where
        Self: Sized + 'static,
    {
        let (id, task) = Message::layershell_open(self.layer_shell_settings());

        (
            Window {
                id,
                inner: Box::new(self),
            },
            task,
        )
    }

    fn is_animating(&self) -> bool {
        false
    }

    fn kind(&self) -> Kind;

    fn layer_shell_settings(&self) -> NewLayerShellSettings;
    fn update(&mut self, id: iced::window::Id, msg: &Message) -> Task<Message>;
    fn view(&self, id: iced::window::Id) -> Element<'_, Message>;
}

impl ShellWindow for Window {
    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn layer_shell_settings(&self) -> NewLayerShellSettings {
        self.inner.layer_shell_settings()
    }

    fn update(&mut self, id: iced::window::Id, msg: &Message) -> Task<Message> {
        self.inner.update(id, msg)
    }

    fn view(&self, id: iced::window::Id) -> Element<'_, Message> {
        self.inner.view(id)
    }

    fn is_animating(&self) -> bool {
        self.inner.is_animating()
    }
}
