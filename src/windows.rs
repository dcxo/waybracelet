use std::ops::{Deref, DerefMut};

use iced::window;

use crate::features::Feature;

#[derive(Debug, Clone)]
pub(super) struct Window<T>
where
    T: Feature,
{
    pub id: window::Id,
    pub view: T,
}

impl<T> Deref for Window<T>
where
    T: Feature,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<T> DerefMut for Window<T>
where
    T: Feature,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view
    }
}
