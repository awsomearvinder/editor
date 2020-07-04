//Everything here is related to Application.
pub struct Editor {
    text_view: TextView,
}

impl iced::Application for Editor {
    type Executor = iced::executor::Default;
    type Message = Msg;
    type Flags = ();
    fn new(_flags: ()) -> (Self, iced::Command<Msg>) {
        (
            Self {
                text_view: TextView::new(),
            },
            iced::Command::none(),
        )
    }
    fn view(&mut self) -> iced::Element<Msg> {
        self.text_view.view()
    }
    fn update(&mut self, event: Msg) -> iced::Command<Msg> {
        self.text_view.update(event)
    }
    fn subscription(&self) -> iced::Subscription<Msg> {
        let app_subscriptions = iced_native::subscription::events().map(Msg::IcedEvent);
        let text_view_subscriptions = self.text_view.subscription();
        iced::Subscription::batch(vec![app_subscriptions, text_view_subscriptions])
    }
    fn title(&self) -> String {
        String::from("Placeholder title")
    }
}

//Everything below here relates to TextView in some form.
pub struct TextView {
    state: CanvasState,
    bridge: crate::bridge::Bridge,
    text: Text,
    background: Background,
}
impl TextView {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            text: Text(Vec::new()),
            background: Background,
            state: CanvasState::new(),
            bridge: crate::bridge::Bridge::new(),
        }
    }
    pub fn view(&mut self) -> iced::Element<Msg> {
        iced::Canvas::new()
            .push(self.state.background_cache.with(&self.background))
            .push(self.state.text_cache.with(&self.text))
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
    pub fn update(&mut self, event: Msg) -> iced::Command<Msg> {
        match event {
            Msg::IcedEvent(iced_native::Event::Window(iced_native::window::Event::Resized {
                width,
                height,
            })) => iced::Command::batch(vec![
                self.bridge.change_resolution(width, height),
                self.bridge.open_file(std::path::PathBuf::from("~/test")),
            ]),
            Msg::IcedEvent(iced_native::Event::Keyboard(event)) => match event {
                iced_native::input::keyboard::Event::CharacterReceived(c) => {
                    eprintln!("sending {}", c);
                    self.bridge.send_input(c)
                }
                _ => iced::Command::none(),
            },
            Msg::BufUpdate(_) => {
                eprintln!("Got a buf update! YES!");
                iced::Command::none()
            }
            _ => iced::Command::none(),
        }
    }
    pub fn subscription(&self) -> iced::Subscription<crate::ui::Msg> {
        iced::Subscription::from_recipe(NvimUpdateHandler(self.bridge.rx.clone()))
    }
}

#[derive(Debug, Default)]
struct CanvasState {
    background_cache: iced::canvas::layer::Cache<Background>,
    text_cache: iced::canvas::layer::Cache<Text>,
}
impl CanvasState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
struct Background;
impl iced::widget::canvas::Drawable for Background {
    fn draw(&self, frame: &mut iced::widget::canvas::Frame) {
        let dimensions = frame.size();
        frame.fill(
            &iced::widget::canvas::Path::rectangle(iced::Point::new(0f32, 0f32), dimensions),
            iced::Color::from_rgb8(27, 43, 52),
        );
    }
}

#[derive(Clone, Debug)]
struct Text(Vec<String>);
impl iced::widget::canvas::Drawable for Text {
    fn draw(&self, frame: &mut iced::widget::canvas::Frame) {
        frame.fill_text(self.0.iter().fold(String::new(), |acc, c| acc + &c))
    }
}
//messages sent into update.
#[derive(Debug)]
pub enum Msg {
    IcedEvent(iced_native::Event),
    BufUpdate(Vec<String>),
    UpdatedResolution(()),
    AttachedUI(()),
    GotBridge(crate::bridge::Bridge),
    OpenedFile(()),
    SentInput(()),
}

pub struct NvimUpdateHandler(
    std::sync::Arc<async_mutex::Mutex<tokio::sync::mpsc::UnboundedReceiver<Msg>>>,
);
impl<H, I> iced_futures::subscription::Recipe<H, I> for NvimUpdateHandler
where
    H: std::hash::Hasher,
{
    type Output = Msg;
    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }
    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(self.0, |rx| async move {
            Some((rx.clone().lock().await.recv().await.unwrap(), rx))
        }))
    }
}
