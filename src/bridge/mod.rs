use nvim_rs::create::tokio as create;

type MyNeovim = nvim_rs::neovim::Neovim<nvim_rs::compat::tokio::Compat<tokio::process::ChildStdin>>;

pub struct Bridge {
    nvim_session: std::sync::Arc<async_mutex::Mutex<MyNeovim>>,
    already_attached_ui: bool,
    pub rx:
        std::sync::Arc<async_mutex::Mutex<tokio::sync::mpsc::UnboundedReceiver<crate::ui::Msg>>>,
}
impl std::fmt::Debug for Bridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bridge")
    }
}
impl Bridge {
    //This is dumb. I acknowledge this is dumb. This is less complex then the
    //alternative of sending a command and managing all that mess.
    //It's blocking on it's futures. Yes. I know. It's bad. It's inefficient.
    //It only runs once. I don't give  a shit.
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let neovim = std::sync::Arc::new(async_mutex::Mutex::new(
            futures::executor::block_on(async {
                create::new_child(Handler(std::sync::Arc::new(std::sync::Mutex::new(tx)))).await
            })
            .unwrap()
            .0,
        ));
        futures::executor::block_on(async { neovim.lock().await.subscribe("buf").await }).unwrap();
        Self {
            nvim_session: neovim,
            already_attached_ui: false,
            rx: std::sync::Arc::new(async_mutex::Mutex::new(rx)),
        }
    }

    pub fn change_resolution(&mut self, width: u32, height: u32) -> iced::Command<crate::ui::Msg> {
        let nvim_session = self.nvim_session.clone();
        if !self.already_attached_ui {
            self.already_attached_ui = true;
            iced::Command::perform(
                async move {
                    nvim_session
                        .lock()
                        .await
                        .ui_attach(
                            (width / 16) as i64,
                            (height / 16) as i64,
                            &nvim_rs::UiAttachOptions::new(),
                        )
                        .await
                        .unwrap_or_else(|e| panic!("Could not attach UI, debug: {}", e));
                },
                crate::ui::Msg::AttachedUI,
            )
        } else {
            iced::Command::perform(
                async move {
                    nvim_session
                        .lock()
                        .await
                        .ui_try_resize(width as i64, height as i64)
                        .await
                        .unwrap_or_else(|e| panic!("failed to resize UI, debug: {}", e));
                },
                crate::ui::Msg::UpdatedResolution,
            )
        }
    }
    pub fn open_file(&self, file: std::path::PathBuf) -> iced::Command<crate::ui::Msg> {
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
            crate::ui::Msg::OpenedFile,
        )
    }
}

#[derive(Clone)]
struct Handler(
    std::sync::Arc<std::sync::Mutex<tokio::sync::mpsc::UnboundedSender<crate::ui::Msg>>>,
);
#[async_trait::async_trait]
impl nvim_rs::Handler for Handler {
    type Writer = nvim_rs::compat::tokio::Compat<tokio::process::ChildStdin>;
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

    async fn handle_notify(&self, _name: String, _args: Vec<rmpv::Value>, _neovim: MyNeovim) {
        self.0
            .lock()
            .unwrap()
            .send(crate::ui::Msg::BufUpdate(Vec::new()))
            .unwrap();
    }
}
