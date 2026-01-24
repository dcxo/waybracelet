use std::{convert::identity, process::Stdio, time::Duration};

use chrono::{Local, Timelike};
use hyprland::{
    data::Workspaces,
    event_listener::{Event, EventStream},
    shared::HyprData,
};
use iced::{Function, Subscription, time};
use smol::{
    io::{AsyncBufRead, AsyncBufReadExt, BufReader},
    process::Command,
    stream::StreamExt,
};

use crate::features::status_bar::StatusBarMessage;

pub(super) fn cava_subscription() -> Subscription<StatusBarMessage> {
    Subscription::run_with(0xCD, |_| {
        let mut command = Command::new("cava");
        command.args(["-p", "/home/dcxo/nixos/dotfiles/quickshell/cava-config"]);

        command.stdout(Stdio::piped());

        let mut child = command.spawn().expect("Could not run CAVA");
        let stdout = child
            .stdout
            .take()
            .expect("Child did not have a handle to stdout");

        let reader = BufReader::new(stdout).lines();

        reader
            .filter_map(Result::ok)
            .map(|line: String| {
                line.split(';')
                    .filter_map(|s| s.parse().ok())
                    .map(|f: f32| f / 1000.)
                    .collect::<_>()
            })
            .skip_while(|v: &Vec<f32>| v.iter().all(|f| *f == 0.))
            .map(StatusBarMessage::CavaInfo)
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
