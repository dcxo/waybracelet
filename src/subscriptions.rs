use std::borrow::Cow;
use std::convert::identity;
use std::process::Stdio;
use std::time::Duration;

use chrono::Local;
use hyprland::event_listener::{Event, EventStream};
use hyprland::{
    data::Workspaces,
    shared::{HyprData, WorkspaceType},
};
use iced::{Subscription, time};
use mpris::PlayerFinder;
use mpris_async::events::PlayerEventsStream;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::LinesStream;

use super::Message;

pub(crate) fn cava_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        let mut command = Command::new("cava");
        command.args(["-p", "/home/dcxo/nixos/dotfiles/quickshell/cava-config"]);

        command.stdout(Stdio::piped());

        let mut child = command.spawn().expect("Could not run CAVA");
        let stdout = child
            .stdout
            .take()
            .expect("Child did not have a handle to stdout");

        let reader = LinesStream::new(BufReader::new(stdout).lines());

        reader
            .filter_map(|line| line.ok())
            .map(|line: String| {
                line.split(';')
                    .filter_map(|s| s.parse().ok())
                    .map(|f: f32| f / 1000.)
                    .collect::<_>()
            })
            .skip_while(|v: &Vec<f32>| v.iter().all(|f| *f == 0.))
            .map(Message::CavaInfo)
    })
}

pub(super) fn playerctl_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        let player_finder = PlayerFinder::new().unwrap();
        let player = player_finder.find_active().unwrap();

        PlayerEventsStream::new(&player)
            .skip_while(|e| match e {
                mpris::Event::Paused => true,
                mpris::Event::Playing => true,
                _ => false,
            })
            .map(|e| match e {
                mpris::Event::Paused => false,
                mpris::Event::Playing => true,
                _ => unreachable!("Others events have been filtered"),
            })
            .map(Message::PlayerState)
    })
}

pub(crate) fn clock_subscription() -> Subscription<Message> {
    time::every(Duration::from_secs_f32(20.))
        .map(|_| Local::now())
        .map(Message::UpdateDatetime)
}

pub fn hyprland_subscription<'a: 'static>(output_name: impl Into<String>) -> Subscription<Message> {
    Subscription::run_with(output_name.into(), |output_name| {
        let stream = EventStream::new();
        let output_name = output_name.clone();

        // FIX: Remove unwrap
        let workspaces = Workspaces::get()
            .unwrap()
            .into_iter()
            .filter(|w| w.monitor == output_name)
            .collect::<Vec<_>>();

        stream
            .filter_map(|event| event.ok())
            .map(move |event| match event {
                Event::WorkspaceChanged(w) if workspaces.iter().any(|cw| cw.id == w.id) => {
                    Some(w.id)
                }
                _ => None,
            })
            .filter_map(identity)
            .map(Message::UpdateCurrenWorkspace)
    })
}
