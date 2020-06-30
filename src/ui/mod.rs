pub struct Editor {
    code_editor_state: CanvasState,
}

impl iced::Sandbox for Editor {
    type Message = Msg;
    fn new() -> Editor {
        Editor {
            code_editor_state: CanvasState {
                background: Background,
                cache: iced::widget::canvas::layer::Cache::new(),
            },
        }
    }
    fn view(&mut self) -> iced::Element<Msg> {
        let canvas = iced::canvas::Canvas::new()
            .push(self.code_editor_state.cache.with(&Background))
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);
        canvas.into()
    }
    fn update(&mut self, _event: Msg) {}
    fn title(&self) -> String {
        String::from("Placeholder title")
    }
}

#[derive(Debug)]
pub enum Msg {}

struct CanvasState {
    pub background: Background,
    cache: iced::canvas::layer::Cache<Background>,
}

#[derive(Debug)]
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
