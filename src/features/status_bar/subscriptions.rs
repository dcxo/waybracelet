use std::{convert::identity, process::Stdio, time::Duration};

use chrono::{Local, Timelike};
use hyprland::{
    data::Workspaces,
    event_listener::{Event, EventStream},
    shared::HyprData,
};
use iced::{Function, Subscription, futures::SinkExt, stream::channel, time};
use smol::{
    io::{AsyncBufRead, AsyncBufReadExt, BufReader},
    process::Command,
    stream::StreamExt,
};

use crate::features::status_bar::StatusBarMessage;

pub(super) fn cava_subscription() -> Subscription<StatusBarMessage> {
    Subscription::run_with(0xCD, |_| {
        let mut command = Command::new("cava");
        command.stdout(Stdio::piped());

        let mut child = command.spawn().expect("Could not run CAVA");
        let stdout = child
            .stdout
            .take()
            .expect("Child did not have a handle to stdout");

        let mut reader = BufReader::new(stdout).lines();

        channel(5, async move |mut sender| {
            while let Some(Ok(line)) = reader.next().await {
                let line = line
                    .split(';')
                    .filter_map(|i| i.parse().ok())
                    .map(|f: f32| f / 1000.)
                    .collect::<Vec<_>>();

                if line.iter().any(|f| *f != 0.) {
                    sender.send(StatusBarMessage::CavaInfo(line)).await.unwrap();
                }
            }
        })
    })
}

// pub(super) fn playerctl_subscription() -> Subscription<StatusBarMessage> {
//     Subscription::run(|| {
//         let player_finder = PlayerFinder::new().unwrap();
//         let player = player_finder.find_active().unwrap();
//         PlayerEventsStream::new(&player)
//             .skip_while(|e| matches!(e, mpris::Event::Paused | mpris::Event::Playing))
//             .map(|e| matches!(e, mpris::Event::Playing))
//             .map(StatusBarMessage::PlayerState)
//     })
// }

pub(super) fn clock_subscription() -> Subscription<StatusBarMessage> {
    time::every(Duration::from_secs_f32(1.))
        .map(|_| Local::now())
        .filter_map(|dt| if dt.second() <= 1 { Some(dt) } else { None })
        .map(StatusBarMessage::UpdateDatetime)
}

pub(super) fn hyprland_subscription<'a: 'static>(
    output_name: impl Into<String>,
) -> Subscription<StatusBarMessage> {
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
            .filter_map(Result::ok)
            .map(move |event| match event {
                Event::WorkspaceChanged(w) if workspaces.iter().any(|cw| cw.id == w.id) => {
                    Some(w.id)
                }
                _ => None,
            })
            .filter_map(identity)
            .map(StatusBarMessage::UpdateCurrenWorkspace.with(output_name))
    })
}
