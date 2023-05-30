use std::fs::DirEntry;
use std::{io, fs};
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::layout::Direction;
use tui::style::Modifier;
use tui::text::Span;
use tui::widgets::{Clear, Padding, ListItem, List, ListState};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};

trait EventHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent);
}

struct AppState {
    running: bool,
    quitting: Option<QuitState>,
    main_menu: MainMenuState,
    component: Box<dyn Component>,
}

impl Default for AppState {
    fn default() -> Self {
        let component: Box<dyn Component> = match OpenFileDialog::new() {
            Ok(dialog) => Box::new(dialog),
            Err(_) => Box::new(MainMenu2::default()),
        };
        Self {
            running: true,
            quitting: None,
            main_menu: MainMenuState::default(),
            component,
        }
    }
}

trait Component {
    fn render(&mut self, area: Rect, buf: &mut Buffer);
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()>;
}

impl EventHandler for AppState {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let Some(quit_state) = self.quitting.as_mut() {
            quit_state.handle_key_event(key_event);
            self.running = quit_state.keep_running;
            if quit_state.exit_modal {
                self.quitting = None;
            }
        } else {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.quitting = Some(QuitState::default()),
                _ => if self.main_menu.active {
                    self.component.handle_key_event(key_event).is_ok();
                }
            }
        }
    }
}

struct App {}

impl App {
    fn default_style() -> Style {
        Style::default().fg(Color::Cyan).bg(Color::Black)
    }
}

impl StatefulWidget for App {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let main_border = Block::default()
            .title("Golem Certificate Manager")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(2, 2, 2, 2))
            .style(App::default_style());

        let main_area = main_border.inner(area);
        main_border.render(area, buf);

        if state.main_menu.active {
            state.component.render(main_area, buf);
        }

        if let Some(quit_state) = state.quitting.as_mut() {
            (Quit {}).render(area, buf, quit_state);
        }
    }
}

struct QuitState {
    quitting: bool,
    keep_running: bool,
    exit_modal: bool,
}

impl Default for QuitState {
    fn default() -> Self {
        Self { quitting: false, keep_running: true, exit_modal: false }
    }
}

impl EventHandler for QuitState {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.exit_modal = true,
            KeyCode::Left => self.quitting = true,
            KeyCode::Right => self.quitting = false,
            KeyCode::Enter => if self.quitting {
                self.keep_running = false;
            } else {
                self.exit_modal = true;
            },
            _ => {},
        }
    }
}

struct Quit {}

impl StatefulWidget for Quit {
    type State = QuitState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let message_box = get_middle_rectangle(area, 7, 19);
        Clear.render(message_box, buf);
        let message_block = Block::default()
            .title("Quit")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(App::default_style());

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Min(1),
            ])
            .split(message_block.inner(message_box));
        message_block.render(message_box, buf);

        Paragraph::new("Are you sure?")
            .style(App::default_style())
            .alignment(Alignment::Center)
            .render(rows[1], buf);

        let choices = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(rows[3]);

        let normal = App::default_style();
        let selected = App::default_style().add_modifier(Modifier::REVERSED);

        Paragraph::new("Quit")
            .style(if state.quitting { selected } else { normal })
            .alignment(Alignment::Center)
            .render(choices[0], buf);

        Paragraph::new("Cancel")
            .style(if state.quitting { normal } else { selected })
            .alignment(Alignment::Center)
            .render(choices[1], buf);
    }
}

struct MainMenuState {
    active: bool,
    selected_item: usize,
    items: Vec<&'static str>,
}

impl Default for MainMenuState {
    fn default() -> Self {
        static MENU_ITEMS: [&str; 6] = [
            "Verify document",
            "",
            "Create node descriptor",
            "Create certificate",
            "",
            "Create key pair",
        ];
        Self {
            active: true,
            selected_item: 0,
            items: MENU_ITEMS.into(),
        }
    }
}

impl EventHandler for MainMenuState {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                }
                while self.selected_item > 0 && self.items[self.selected_item].len() < 1 {
                    self.selected_item -= 1;
                }
            },
            KeyCode::Down => {
                let max = self.items.len() - 1;
                if self.selected_item < max {
                    self.selected_item += 1;
                }
                while self.selected_item < max && self.items[self.selected_item].len() < 1 {
                    self.selected_item += 1;
                }
            },
            KeyCode::Enter => self.active = false,
            _ => {}
        }
    }
}

struct MainMenu {}

impl StatefulWidget for MainMenu {
    type State = MainMenuState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {

        let longest_item = state.items.iter().map(|i| i.len() as u16).max().unwrap();

        let menu_area = get_middle_rectangle(area, state.items.len() as u16, longest_item);

        let row_constraints = {
            let mut constraints = vec![Constraint::Max(1); state.items.len()];
            constraints.push(Constraint::Min(0));
            constraints
        };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(menu_area);

        let normal = App::default_style();
        let selected = App::default_style().add_modifier(Modifier::REVERSED);

        state.items.iter().enumerate().for_each(|(idx, &item)| {
            if item.len() > 0 {
                Paragraph::new(item)
                    .style(if state.selected_item == idx { selected } else { normal })
                    .alignment(Alignment::Center)
                    .render(rows[idx], buf);
            }
        })
    }
}

struct MainMenu2 {
    state: MainMenuState,
}

impl Component for MainMenu2 {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        (MainMenu {}).render(area, buf, &mut self.state)
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        self.state.handle_key_event(key_event);
        Ok(())
    }
}

impl Default for MainMenu2 {
    fn default() -> Self {
        Self { state: Default::default() }
    }
}

fn get_middle_rectangle(area: Rect, height: u16, width: u16) -> Rect {
    let horizontal_border = (area.height - height) / 2;
    let vertical_border = (area.width - width) / 2;
    let row = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(horizontal_border),
            Constraint::Min(height),
            Constraint::Max(horizontal_border),
        ])
        .split(area)[1];
    let message_box = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Max(vertical_border),
            Constraint::Min(width),
            Constraint::Max(vertical_border),
        ])
        .split(row)[1];
    message_box
}

#[derive(Default)]
struct OpenFileDialog {
    current_directory: PathBuf,
    file_names: Vec<std::ffi::OsString>,
    list_state: ListState,
}

impl OpenFileDialog {
    fn new() -> Result<Self> {
        let mut dialog = OpenFileDialog::default();
        let current_directory = std::env::current_dir()?;
        dialog.set_directory(current_directory)?;
        Ok(dialog)
    }

    fn go_to_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.current_directory.parent() {
            let directory_name = self.current_directory.file_name()
                .map(|filename| filename.to_owned())
                .ok_or_else(|| anyhow::anyhow!("Some error happened reading the filename of current directory"))?;
            self.set_directory(parent.to_path_buf())?;
            let previous_directory_index = self.file_names.iter().enumerate()
                .find_map(|(idx, file_name)| if *file_name == directory_name { Some(idx) } else { None });
            self.list_state = ListState::default().with_selected(previous_directory_index.or(Some(0)));
        }
        Ok(())
    }

    fn set_directory(&mut self, directory: PathBuf) -> Result<()> {
        self.current_directory = directory.canonicalize()?;
        self.file_names = fs::read_dir(directory)?
            .filter_map(|res| res.map(|entry| entry.file_name()).ok())
            .collect::<Vec<_>>();
        self.file_names.sort();
        self.file_names.insert(0, std::ffi::OsStr::new("..").into());
        self.list_state = ListState::default().with_selected(Some(0));
        Ok(())
    }
}

impl Component for OpenFileDialog {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::raw(self.current_directory.to_string_lossy()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(App::default_style());
        let list_area = block.inner(area);
        block.render(area, buf);

        let list_parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(1),
                Constraint::Min(1),
                Constraint::Max(1),
            ])
            .split(list_area);

        let list_items = self.file_names.iter()
            .map(|entry| ListItem::new(entry.to_string_lossy()))
            .collect::<Vec<_>>();

        let list = List::new(list_items)
            .style(App::default_style())
            .highlight_style(App::default_style().add_modifier(Modifier::REVERSED));
        StatefulWidget::render(list, list_parts[1], buf, &mut self.list_state);
        if self.list_state.offset() > 0 {
            Paragraph::new("  ^^^^^")
                .style(App::default_style())
                .render(list_parts[0], buf)
        }
        if (self.list_state.offset() + list_parts[1].height as usize) < self.file_names.len() {
            Paragraph::new("  vvvvv")
                .style(App::default_style())
                .render(list_parts[2], buf)
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Up => {
                if let Some(idx) = self.list_state.selected().and_then(|i| i.checked_sub(1)) {
                    self.list_state.select(Some(idx));
                }
            }
            KeyCode::Down => {
                if let Some(idx) = self.list_state.selected() {
                    if idx < self.file_names.len() - 1 {
                        self.list_state.select(Some(idx + 1));
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(idx) = self.list_state.selected() {
                    let path = {
                        let mut path = self.current_directory.clone();
                        path.push(&self.file_names[idx]);
                        path.canonicalize()?
                    };
                    if path.is_dir() {
                        self.set_directory(path)?;
                    } else {
                        println!("Selected file {}", path.to_string_lossy());
                    }
                }
            }
            KeyCode::Backspace => {
                self.go_to_parent()?;
            }
            _ => {}
        }
        Ok(())
    }
}



pub fn start() -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::default();

    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    terminal.clear()?;

    while app.running {
        terminal.draw(|frame| frame.render_stateful_widget(App {}, frame.size(), &mut app))?;
        match event::read()? {
            Event::Key(e) => {
                if (e.code == KeyCode::Char('c') || e.code == KeyCode::Char('C')) && e.modifiers == KeyModifiers::CONTROL {
                    app.running = false;
                } else {
                    app.handle_key_event(e);
                }
            },
            Event::Mouse(e) => println!("{:?}", e),
            Event::Resize(_w, _h) => {},
            _ => unimplemented!()
        }
    }


    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
