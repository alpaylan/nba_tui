/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Write;

use std::env;


pub mod positions;

use crate::positions::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Player {
    name: String,
    team: String,
    #[serde(rename = "position")]
    position: Vec<Position>,
    pick_avg: f32,
    round_avg: f32,
    draft_percent: String,
}


#[derive(Eq, PartialEq, Debug)]
enum InputMode {
    Idle,
    Searching,
    Picking,
    Listing,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// List of all players
    all_players: Vec<Player>,
    /// My players
    my_players: Vec<String>,
    /// Other's players
    other_players: Vec<String>,
    /// Filtered list of players
    filtered_players: Vec<String>,
    /// Current selected player
    selected_player: Option<usize>,
    /// Candidate player
    candidate_player: String,
    /// selected position
    selected_position: Position,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Idle,
            all_players: Vec::new(),
            my_players: Vec::new(),
            other_players: Vec::new(),
            filtered_players: Vec::new(),
            selected_player: None,
            candidate_player: String::new(),
            selected_position: Position::ANY,
        }
    }
}

impl App {
    fn filter_players(&mut self) {
        self.filtered_players = self
            .all_players
            .iter()
            .filter(|p| 
                p.name.to_ascii_lowercase().contains(&self.input.to_ascii_lowercase()) 
                && !self.my_players.contains(&p.name) 
                && !self.other_players.contains(&p.name)
                && p.position
                        .iter()
                        .any(|x| x.does_position_belong(&self.selected_position))
            )
            .take(8)
            .cloned()
            .map(|p| p.name)
            .collect();
    }

    fn get_player(&self, name: &String) -> Option<&Player> {
        self.all_players.iter().find(|p| p.name == *name)
    }

    fn save_players(&self, players: &Vec<String>, filename: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(filename)?;
        let players = players.clone();
        let json = serde_json::to_string(&players)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn slots() -> Vec<(Position, u16)> {
        vec![
            (Position::C, 3),
            (Position::PF, 1),
            (Position::PG, 1),
            (Position::SG, 1),
            (Position::SF, 1),
            (Position::G, 1),
            (Position::F, 1),
            (Position::ANY, 7),
        ]
    }

}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // load players
    let file = File::open("data.json")?;
    
    // use seerde_json to deserialize the JSON data
    let players: Vec<Player> = serde_json::from_reader(file)?;
    
    // create app and run it
    let mut app = App::default();

    app.all_players = Vec::new();
    for player in players {
        app.all_players.push(player);
    }

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "load" {
            // check if my_players.json exists
            let my_players_file = File::open("my_players.json");
            if let Ok(file) = my_players_file {
                let my_players: Vec<String> = serde_json::from_reader(file)?;
                app.my_players = my_players;
            }

            let other_players_file = File::open("other_players.json");
            if let Ok(file) = other_players_file {
                let other_players: Vec<String> = serde_json::from_reader(file)?;
                app.other_players = other_players;
            }
        } else if args[1] == "delete" {
            let my_players_file = File::open("my_players.json");
            if let Ok(_) = my_players_file {
                std::fs::remove_file("my_players.json")?;
            }
            let my_players_file = File::open("other_players.json");
            if let Ok(_) = my_players_file {
                std::fs::remove_file("other_players.json")?;
            }
        }
    }

    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Right {
                app.selected_position = match app.selected_position {
                    Position::ANY => Position::PG,
                    Position::PG => Position::SG,
                    Position::SG => Position::SF,
                    Position::SF => Position::PF,
                    Position::PF => Position::C,
                    Position::C => Position::F,
                    Position::F => Position::G,
                    Position::G => Position::TALL,
                    Position::TALL => Position::SHORT,
                    Position::SHORT => Position::ANY,
                };
                app.filter_players();
            } else if key.code == KeyCode::Left {
                app.selected_position = match app.selected_position {
                    Position::ANY => Position::SHORT,
                    Position::PG => Position::ANY,
                    Position::SG => Position::PG,
                    Position::SF => Position::SG,
                    Position::PF => Position::SF,
                    Position::C => Position::PF,
                    Position::F => Position::C,
                    Position::G => Position::F,
                    Position::TALL => Position::G,
                    Position::SHORT => Position::TALL,
                };
                app.filter_players();
            }
            match app.input_mode {
                InputMode::Idle => match key.code {
                    KeyCode::Char('s') | KeyCode::Enter | KeyCode::Up | KeyCode::Down => {
                        app.input_mode = InputMode::Searching;
                        app.filter_players();
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('l') => {
                        app.input_mode = InputMode::Listing;
                    }
                    _ => {}
                },
                InputMode::Searching => match key.code {
                    KeyCode::Enter => {
                        if let Some(selected) = app.selected_player {
                            app.candidate_player = app.filtered_players[selected].clone();
                            app.input_mode = InputMode::Picking;
                        } else {
                            if app.filtered_players.len() > 0 {
                                app.selected_player = Some(0);
                                app.input = app.filtered_players[0].clone();
                                app.filter_players();
                            }
                        }
                    }
                    KeyCode::Tab => {
                        if app.filtered_players.len() > 0 {
                            app.selected_player = Some(0);
                            app.input = app.filtered_players[0].clone();
                            app.filter_players();
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = app.selected_player {
                            if selected > 0 {
                                app.selected_player = Some(selected - 1);
                            }
                        }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = app.selected_player {
                            if selected < app.filtered_players.len() - 1 {
                                app.selected_player = Some(selected + 1);
                            }
                        } else {
                            if !app.filtered_players.is_empty() {
                                app.selected_player = Some(0);
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if c.is_ascii_digit() {
                            let c = c.to_digit(10).unwrap() as usize;
                            if c <= app.filtered_players.len() {
                                app.selected_player = Some(0);
                                app.input = app.filtered_players[c - 1].clone();
                                app.filter_players();
                            }
                        } else {
                            app.input.push(c);
                            app.filter_players();
                        }
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.filter_players();
                    }
                    KeyCode::Esc => {
                        app.candidate_player.clear();
                        app.input.clear();
                        app.filter_players();
                        app.selected_player = None;
                        app.input_mode = InputMode::Idle;
                    }
                    _ => {}
                },
                InputMode::Picking => match key.code {
                    KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Enter => {
                        app.my_players.push(app.candidate_player.clone());
                        app.save_players(&app.my_players, "my_players.json").unwrap();
                        app.candidate_player.clear();
                        app.input.clear();
                        app.filter_players();
                        app.selected_player = None;
                        app.input_mode = InputMode::Searching;
                    }
                    KeyCode::Char('b') | KeyCode::Char('B') => {
                        app.other_players.push(app.candidate_player.clone());
                        app.save_players(&app.other_players, "other_players.json").unwrap();
                        app.candidate_player.clear();
                        app.input.clear();
                        app.filter_players();
                        app.selected_player = None;
                        app.input_mode = InputMode::Searching;
                    }
                    KeyCode::Esc => {
                        app.candidate_player.clear();
                        app.input.clear();
                        app.filter_players();
                        app.selected_player = None;
                        app.input_mode = InputMode::Searching;
                    }
                    _ => {}
                },
                InputMode::Listing => match key.code {
                    KeyCode::Char('q') => {
                        app.input_mode = InputMode::Idle;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Idle => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("s or Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start searching,"),
                Span::styled("l", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start listing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Searching => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop searching, "),
                Span::styled("Up/Down Arrows", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to select player,"),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to pick the player"),
            ],
            Style::default(),
        ),
        InputMode::Picking => (
            vec![
                Span::raw("Press "),
                Span::styled("A or Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to add to my team, "),
                Span::styled("B", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to add to other team,"),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to go back to searching"),
            ],
            Style::default(),
        ),
        InputMode::Listing => (
            vec![
                Span::raw("Press "),
                Span::styled("Q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to go back to idle "),
            ],
            Style::default(),
        )
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Idle => Style::default(),
            InputMode::Searching => Style::default().fg(Color::Yellow),
            InputMode::Picking => Style::default().fg(Color::Blue),
            InputMode::Listing => Style::default().fg(Color::Red),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Idle =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Searching => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
        InputMode::Picking => {}
        InputMode::Listing => {}
    }

    let (player_set, title) = match app.input_mode {
        InputMode::Idle => (&app.filtered_players, "Doing nothing"),
        InputMode::Searching => (&app.filtered_players, "Searching players"),
        InputMode::Picking => (&app.filtered_players, "Picking a player"),
        InputMode::Listing => (&app.my_players, "My players"),
    };
    if app.input_mode != InputMode::Listing {
        let players: Vec<ListItem> = player_set
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let player: &Player = app.get_player(m).unwrap();
                let content = vec![Spans::from(Span::raw(format!("{}: {} {:?}", i + 1, player.name, player.position)))];
                let color = match app.input_mode {
                    InputMode::Idle | InputMode::Listing => Color::Reset,
                    InputMode::Searching => {
                        if Some(i) == app.selected_player {
                            Color::Yellow
                        } else {
                            Color::Reset
                        }
                    }
                    InputMode::Picking => {
                        if Some(i) == app.selected_player {
                            Color::Blue
                        } else {
                            Color::Reset
                        }
                    }
                };
                ListItem::new(content).style(Style::default().fg(color))
                
            })
            .collect();

        let players = List::new(players).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(players, chunks[2]);
    } else {
        let slots = App::slots();
        let mut filled_slots: Vec<(Position, String, Vec<Position>)> = Vec::new();

        for (position, slot) in slots.iter() {
            let mut slots_left = slot.clone();
            for player in app.my_players.iter() {
                let player: &Player = app.get_player(player).unwrap();
                if  filled_slots.iter().find(|x| x.1 == player.name).is_none() &&
                    player.position.iter().any(|p| p.does_position_belong(position)) {
                    if slots_left > 0 {
                        filled_slots.push((position.clone(), player.name.clone(), player.position.clone()));
                        slots_left -= 1;
                    }
                }
                if slots_left == 0 {
                    break;
                }
            }
            while slots_left > 0 {
                filled_slots.push((position.clone(), "Empty".to_string(), vec![]));
                slots_left -= 1;
            }
        }

        let players: Vec<ListItem> = filled_slots
            .iter()
            .map(|(position, name, player_position)| {
                let content = vec![Spans::from(Span::raw(format!("{:?}: {} {:?}", position, name, player_position)))];
                let color = if name == "Empty" {
                    Color::Red
                } else {
                    if player_position.len() == 1 {
                        Color::Green
                    } else {
                        Color::Yellow
                    }
                };
                ListItem::new(content).style(Style::default().fg(color))
                
            })
            .collect();

        let players = List::new(players).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(players, chunks[2]);
    }
    
    


    // split chunks[3] into 10 chunks, one for each position
    let position_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(chunks[3]);

    for (i, position) in Position::get_all_positions().iter().enumerate() {
        let style = if app.selected_position == *position {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let widget = Paragraph::new(format!("{:?}", position))
            .style(style)
            .block(Block::default().borders(Borders::ALL)
            .title("Pos")
        );
        f.render_widget(widget, position_chunks[i]);
    };
    
}