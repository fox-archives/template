use std::{
	error::Error,
	io, process,
	time::{Duration, Instant},
};

use anyhow::Result;

use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

struct StatefulList<T> {
	state: ListState,
	items: Vec<T>,
}

impl<T> StatefulList<T> {
	fn with_items(items: Vec<T>) -> StatefulList<T> {
		StatefulList {
			state: ListState::default(),
			items,
		}
	}

	fn next(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i >= self.items.len() - 1 {
					0
				} else {
					i + 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}

	fn previous(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i == 0 {
					self.items.len() - 1
				} else {
					i - 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is
/// a wrapper around `ListState`. Keeping track of the items state let us render the associated
/// widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App<'a> {
	a: &'a str,
	items: StatefulList<String>,
}

impl<'a> App<'a> {
	fn from(items: Vec<String>) -> App<'a> {
		App {
			a: "",
			items: StatefulList::with_items(items),
		}
	}
}

fn run_app<B: Backend>(
	terminal: &mut Terminal<B>,
	mut app: App,
	tick_rate: Duration,
) -> Result<String> {
	let last_tick = Instant::now();
	app.items.next();
	loop {
		terminal.draw(|f| ui(f, &mut app))?;

		let timeout = tick_rate
			.checked_sub(last_tick.elapsed())
			.unwrap_or_else(|| Duration::from_secs(0));
		if crossterm::event::poll(timeout)? {
			if let Event::Key(key) = event::read()? {
				if key.kind == KeyEventKind::Press {
					match key.code {
						KeyCode::Char('q') => return Ok("".to_string()),
						KeyCode::Enter => {
							if let Some(i) = app.items.state.selected() {
								let value = app.items.items[i].clone();

								return Ok(value);
							}
						}
						KeyCode::Char('j') => app.items.next(),
						KeyCode::Char('k') => app.items.previous(),
						KeyCode::Down => app.items.next(),
						KeyCode::Up => app.items.previous(),
						_ => {}
					}
				}
			}
		}
	}
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
	// Create two chunks
	let chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
		.split(f.size());

	// Iterate through all elements in the `items` app and append some debug text to it.
	let items: Vec<ListItem> = app
		.items
		.items
		.iter()
		.map(|item| ListItem::new(item.as_str()).style(Style::default()))
		.collect();

	// Create a List from all list items and highlight the currently selected one
	let items = List::new(items)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title("Choose a Template"),
		)
		.highlight_style(
			Style::default()
				.bg(Color::Cyan)
				.add_modifier(Modifier::BOLD),
		)
		.highlight_symbol("> ");

	// We can now render the item list
	f.render_stateful_widget(items, chunks[0], &mut app.items.state);
}

pub fn get_template_name(names: Vec<String>) -> Result<String, Box<dyn Error>> {
	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// create app and run it
	let tick_rate = Duration::from_millis(250);
	let app = App::from(names);
	let res = run_app(&mut terminal, app, tick_rate);

	// restore terminal
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;

	match res {
		Err(err) => {
			println!("{err:?}");
			process::exit(1);
		}
		Ok(val) => Ok(val),
	}
}
