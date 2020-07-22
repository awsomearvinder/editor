/*
 * IMPORTANT:
 * The communication with the bridge should *never* panic,
 * Any panics here are a bug. The UI should be informed of any misbehavior
 * so it can be displayed to the user instead.
 */

use crate::ui::Msg;
//I doubt we plan on using a traditional std::sync::Mutex, so I believe confusion won't exist.
use async_mutex::Mutex;
use futures::executor;
use nvim_rs::{compat::tokio::Compat, create::tokio as create, neovim::Neovim};
use std::{
    convert::TryFrom,
    fmt::{Debug, Formatter},
    sync::Arc,
};
use tokio::{
    process::ChildStdin,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

mod character;
mod errors;
mod grid_line;

type MyNeovim = Neovim<Compat<ChildStdin>>;

//TODO: make this configurable
const FONTSIZE: u32 = 16;

pub struct Bridge {
    nvim_session: Arc<Mutex<MyNeovim>>,
    already_attached_ui: bool,
    pub rx: Arc<Mutex<UnboundedReceiver<Msg>>>,
}

impl Debug for Bridge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bridge")
    }
}

impl Bridge {
    //This is dumb. I acknowledge this is dumb. This is less complex then the
    //alternative of sending a command and managing all that mess.
    //It's blocking on it's futures. Yes. I know. It's bad. It's inefficient.
    //It only runs once. I don't give  a shit.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let (neovim, _, _) =
            executor::block_on(async { create::new_child(Handler::new(tx)).await }).unwrap();

        Self {
            nvim_session: Arc::new(Mutex::new(neovim)),
            already_attached_ui: false,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    ///Changes the resolution reported to neovim - this is a requirement imposed by neovim itself.
    pub fn change_resolution(&mut self, width: u32, height: u32) -> iced::Command<Msg> {
        let nvim_session = self.nvim_session.clone();

        if self.already_attached_ui {
            iced::Command::perform(
                async move {
                    nvim_session
                        .lock()
                        .await
                        .ui_try_resize(width as i64, height as i64)
                        .await
                        .unwrap_or_else(|e| panic!("failed to resize UI, debug: {}", e));
                },
                Msg::UpdatedResolution,
            )
        } else {
            self.already_attached_ui = true;
            iced::Command::perform(
                async move {
                    nvim_session
                        .lock()
                        .await
                        .ui_attach(
                            (width / FONTSIZE) as i64,
                            (height / FONTSIZE) as i64,
                            &nvim_rs::UiAttachOptions::new().set_linegrid_external(true),
                        )
                        .await
                        .unwrap_or_else(|e| panic!("Could not attach UI, debug: {}", e));
                },
                Msg::AttachedUI,
            )
        }
    }

    pub fn open_file(&self, file: std::path::PathBuf) -> iced::Command<Msg> {
        let nvim_session = self.nvim_session.clone();
        iced::Command::perform(
            async move {
                nvim_session
                    .lock()
                    .await
                    .command(&format!("e {}", &file.to_str().unwrap()))
                    .await
                    .unwrap();
            },
            Msg::OpenedFile,
        )
    }

    pub fn send_input(&self, c: char) -> iced::Command<Msg> {
        let nvim_session = self.nvim_session.clone();
        iced::Command::perform(
            async move {
                nvim_session
                    .lock()
                    .await
                    .input(&format!("{}", c))
                    .await
                    .unwrap();
            },
            Msg::SentInput,
        )
    }
}

///This is what recives messages from neovim. look in handle_request
///and handle_notify in nvim_rs::Handler for more information.
#[derive(Clone)]
struct Handler {
    sender_channel: Arc<Mutex<UnboundedSender<Msg>>>,
}

impl Handler {
    fn new(sender: UnboundedSender<Msg>) -> Self {
        Self {
            sender_channel: Arc::new(Mutex::new(sender)),
        }
    }
}

#[async_trait::async_trait]
impl nvim_rs::Handler for Handler {
    type Writer = Compat<ChildStdin>;

    async fn handle_request(
        &self,
        _name: String,
        _args: Vec<rmpv::Value>,
        _neovim: MyNeovim,
    ) -> Result<rmpv::Value, rmpv::Value> {
        eprintln!("ran handler!");
        eprintln!("{}", _name);
        eprintln!("{:?}", _args);
        Ok(rmpv::Value::Nil)
    }

    async fn handle_notify(&self, _name: String, args: Vec<rmpv::Value>, _neovim: MyNeovim) {
        //TODO: Need to figure out how message sending works. T is an enum we send?
        let _channel = self.sender_channel.lock().await;
        for arg in args {
            if !arg.is_array() {
                break;
            }
            let arr = arg.as_array().unwrap();
            if arr.is_empty() {
                break;
            }
            if arr[0].is_str() {
                let name_of_arr = arr[0].as_str();
                if name_of_arr.unwrap() == "grid_line" {
                    let _grid_line = grid_line::GridLine::try_from(&arr[1]).unwrap();
                }
            }
        }
    }
}
