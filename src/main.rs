use crossterm::event::{self, Event, KeyCode, KeyEvent};
use image::{DynamicImage, ImageReader};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    macros::vertical,
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Row, Table, Widget},
};
use ratatui_image::{
    StatefulImage,
    picker::{self, Picker},
    protocol::StatefulProtocol,
};

fn main() {
    let image = ImageReader::open("assets/arc-de-triomphe.png")
        .unwrap()
        .decode()
        .unwrap();
    let mut app = App::new(image);
    ratatui::run(|terminal| app.run(terminal));
}
struct App {
    hue_rotation: i32,
    brightness_adjustment: i32,
    contrast_adjustment: f32,
    blur: f32,
    sharpness_sigma: f32,
    sharpness_threshold: i32,

    exit: bool,
    up_to_date: bool,
    mode: Mode,

    picker: Picker,
    source_image: DynamicImage,
    original_image: StatefulProtocol,
    modified_image: StatefulProtocol,
}
#[derive(PartialEq)]
enum Mode {
    Hue,
    Brightness,
    Contrast,
    Blur,
    SharpnessSigma,
    SharpnessThreshold,
}
impl Mode {
    fn next(&self) -> Self {
        match self {
            Mode::Hue => Mode::Brightness,
            Mode::Brightness => Mode::Contrast,
            Mode::Contrast => Mode::Blur,
            Mode::Blur => Mode::SharpnessSigma,
            Mode::SharpnessSigma => Mode::SharpnessThreshold,
            Mode::SharpnessThreshold => Mode::Hue,
        }
    }
    fn prev(&self) -> Self {
        match self {
            Mode::SharpnessThreshold => Mode::SharpnessSigma,
            Mode::SharpnessSigma => Mode::Blur,
            Mode::Blur => Mode::Contrast,
            Mode::Contrast => Mode::Brightness,
            Mode::Brightness => Mode::Hue,
            Mode::Hue => Mode::SharpnessThreshold,
        }
    }
}

impl App {
    fn new(image: DynamicImage) -> Self {
        let picker: Picker = Picker::from_query_stdio().unwrap();
        let original_image = picker.new_resize_protocol(image.clone());
        let modified_image = picker.new_resize_protocol(image.clone());
        App {
            hue_rotation: 0,
            brightness_adjustment: 0,
            contrast_adjustment: 0.0,
            blur: 0.0,
            sharpness_sigma: 0.0,
            sharpness_threshold: 0,

            mode: Mode::Hue,
            exit: false,
            up_to_date: true,
            picker,
            source_image: image,
            original_image,
            modified_image,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) {
        while !self.exit {
            let _ = terminal.draw(|frame| self.draw(frame));
            self.handle_events();
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let title = Line::from(format!(" Hue Rotator: {}° ", self.hue_rotation));
        let mut block = Block::bordered()
            .border_type(BorderType::Double)
            .title(title.centered());
        let update_reminder;
        if !self.up_to_date {
            update_reminder = Line::from(" Press Enter to Update Rotation ");
        } else {
            update_reminder = Line::from(" Press Enter to Lock In Changes ");
        }
        block = block.title_bottom(update_reminder.centered());

        let main_area = block.inner(frame.area());

        let vertical_layout = vertical![*=1, == 30 %];
        let image_section = vertical_layout.areas::<2>(main_area)[0];
        let editor_section = vertical_layout.areas::<2>(main_area)[1];

        let editor_block = Block::bordered().border_type(BorderType::Thick);
        let editor_block_area = editor_block.inner(editor_section);
        let rows = [
            Row::new([
                "Hue Rotation".to_string(),
                format!("{}°", self.hue_rotation),
            ])
            .style(self.table_style(Mode::Hue)),
            Row::new([
                "Brightness".to_string(),
                format!("{}", self.brightness_adjustment),
            ])
            .style(self.table_style(Mode::Brightness)),
            Row::new([
                "Contrast".to_string(),
                format!("{}", self.contrast_adjustment),
            ])
            .style(self.table_style(Mode::Contrast)),
            Row::new(["Blur".to_string(), format!("{}", self.blur)])
                .style(self.table_style(Mode::Blur)),
            Row::new([
                "Sharpness Sigma".to_string(),
                format!("{}", self.sharpness_sigma),
            ])
            .style(self.table_style(Mode::SharpnessSigma)),
            Row::new([
                "Sharpness Threshold".to_string(),
                format!("{}", self.sharpness_threshold),
            ])
            .style(self.table_style(Mode::SharpnessThreshold)),
        ];
        let cell_widths = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let settings_table = Table::new(rows, cell_widths);

        let image_layout =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let original_zone = image_layout.areas::<2>(image_section)[0];
        let modified_zone = image_layout.areas::<2>(image_section)[1];

        let original_title = Line::from(" Original ");
        let original_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(original_title.centered());
        let original_area = original_block.inner(original_zone);

        let modified_title = Line::from(" Modified ");
        let modified_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(modified_title.centered());
        let modified_area = modified_block.inner(modified_zone);

        let original_widget = StatefulImage::default();
        let modified_widget = StatefulImage::default();

        block.render(frame.area(), frame.buffer_mut());
        editor_block.render(editor_section, frame.buffer_mut());
        settings_table.render(editor_block_area, frame.buffer_mut());
        original_block.render(original_zone, frame.buffer_mut());
        modified_block.render(modified_zone, frame.buffer_mut());
        frame.render_stateful_widget(original_widget, original_area, &mut self.original_image);
        frame.render_stateful_widget(modified_widget, modified_area, &mut self.modified_image);
    }

    fn handle_events(&mut self) {
        match event::read().unwrap() {
            Event::Key(keypress) if keypress.is_press() => self.handle_keypress(keypress),
            _ => {}
        }
    }

    fn handle_keypress(&mut self, keypress: KeyEvent) {
        match keypress.code {
            KeyCode::Left => {
                self.decrease_current_setting();
                self.up_to_date = false;
            }
            KeyCode::Right => {
                self.increase_current_setting();
                self.up_to_date = false;
            }
            KeyCode::Up => {
                self.mode = self.mode.prev();
            }
            KeyCode::Down => {
                self.mode = self.mode.next();
            }
            KeyCode::Esc => self.exit = true,
            KeyCode::Enter => {
                if self.up_to_date {
                    self.save_changes();
                } else {
                    self.regenerate_images();
                }
                self.up_to_date = true;
            }
            _ => {}
        }
    }

    fn regenerate_images(&mut self) {
        self.modified_image = if self.sharpness_sigma != 0.0 {
            self.picker.new_resize_protocol(
                self.source_image
                    .huerotate(self.hue_rotation)
                    .brighten(self.brightness_adjustment)
                    .adjust_contrast(self.contrast_adjustment)
                    .blur(self.blur)
                    .unsharpen(self.sharpness_sigma, self.sharpness_threshold),
            )
        } else {
            self.picker.new_resize_protocol(
                self.source_image
                    .huerotate(self.hue_rotation)
                    .brighten(self.brightness_adjustment)
                    .adjust_contrast(self.contrast_adjustment)
                    .blur(self.blur),
            )
        };
    }

    fn save_changes(&mut self) {
        self.source_image = self.source_image.huerotate(self.hue_rotation);
        self.original_image = self.picker.new_resize_protocol(self.source_image.clone());
        self.modified_image = self.picker.new_resize_protocol(self.source_image.clone());
        self.hue_rotation = 0;
        self.brightness_adjustment = 0;
        self.contrast_adjustment = 0.0;
        self.blur = 0.0;
    }

    fn table_style(&self, row_mode: Mode) -> Style {
        if self.mode == row_mode {
            Style::default().bold().blue()
        } else {
            Style::default()
        }
    }

    fn increase_current_setting(&mut self) {
        match self.mode {
            Mode::Hue => self.hue_rotation += 5,
            Mode::Brightness => self.brightness_adjustment += 5,
            Mode::Contrast => self.contrast_adjustment += 5.0,
            Mode::Blur => self.blur += 5.0,
            Mode::SharpnessSigma => self.sharpness_sigma += 5.0,
            Mode::SharpnessThreshold => self.sharpness_threshold += 5,
        }
    }

    fn decrease_current_setting(&mut self) {
        match self.mode {
            Mode::Hue => self.hue_rotation -= 5,
            Mode::Brightness => self.brightness_adjustment -= 5,
            Mode::Contrast => self.contrast_adjustment -= 5.0,
            Mode::Blur => self.blur -= if self.blur >= 5.0 { 5.0 } else { 0.0 },
            Mode::SharpnessSigma => {
                self.sharpness_sigma -= if self.sharpness_sigma >= 5.0 {
                    5.0
                } else {
                    0.0
                }
            }
            Mode::SharpnessThreshold => self.sharpness_threshold -= 5,
        }
    }
}
