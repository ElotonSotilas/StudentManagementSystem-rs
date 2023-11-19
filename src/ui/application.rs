use iced::Sandbox;

struct Application;

#[derive(Debug)]
enum Message {}

impl Sandbox for Application {
    type Message = Message;

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("Student Management System")
    }

    fn update(&mut self, message: Self::Message) {
        match message {}
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        todo!()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::default()
    }

    fn style(&self) -> iced::theme::Application {
        iced::theme::Application::default()
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }

    fn run(settings: iced::Settings<()>) -> Result<(), iced::Error>
    where
        Self: 'static + Sized,
    {
        <Self as iced::Application>::run(settings)
    }
}
