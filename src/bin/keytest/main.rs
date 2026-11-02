use std::cell::RefCell;
use std::rc::Rc;

use shawtage::data::words;
use shawtage::data::words::Words;

use crossterm::{
    cursor::MoveTo,
    style::Print,
    //Result,
};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, terminal,
};
use std::io::{self, stdout, Write};
use std::time::{Duration, Instant};

use std::collections::{HashMap, HashSet, VecDeque};

mod jafl;
use jafl::Shaw;

struct Pos {
    row: u16,
    col: u16,
}
struct State {
    line: String,
    typed: Vec<char>,
    partial_char: Option<char>,
    line_pos: Pos,
    typed_pos: Pos,
    partial_pos: Pos,
}
impl State {

    fn new(chars: Vec<char>, needs_improvement: Vec<char>) -> Self {
        let char_set: HashSet<char> = chars.iter().cloned().collect();
        let mut words = Words::dictionary(char_set);
        let least_accurate = needs_improvement.iter().cloned().collect();
        const CHARS_PER_LINE: usize = 52;

        Self {
            line: words.line(CHARS_PER_LINE, &least_accurate),
            typed: Vec::new(),
            partial_char: None,
            line_pos: Pos{row: 3, col: 10},
            typed_pos: Pos{row: 4, col: 10},
            partial_pos: Pos{row: 5, col: 10},
        }
    }

    fn _display(&self, pos: &Pos, s: &String) {
        let mut stdout = io::stdout();

        let result = execute!(
            stdout,
            MoveTo(pos.col, pos.row),
            Print(s)
        );
    }

    fn update_display(&self)
    {
        let typed_s: String = self.typed.clone().into_iter().collect();
        let partial_s: String = self.partial_char.into_iter().collect();
        self._display(&self.line_pos, &self.line);
        self._display(&self.typed_pos, &typed_s);
        self._display(&self.partial_pos, &partial_s);
    }

    fn partial(&mut self, c: char) {
        self.partial_char = Some(c);
    }

    fn push(&mut self, c: char) {
        self.partial_char = None;
        self.typed.push(c);
    }

    fn pop(&mut self, c: char) -> Option<char> {
        if self.partial_char.is_none() {
            return self.typed.pop();
        }
        else {
            let result = self.partial_char;
            self.partial_char = None;
            return result;
        }
    }

}

fn main() -> io::Result<()> {
    let chars = vec!['𐑨', '𐑩', '𐑦', '𐑳', '𐑤', '𐑮', '𐑕', '𐑯',
                '𐑧', '𐑰', '𐑥', '𐑑', '𐑛', '𐑓', '𐑾', '𐑽', '𐑼', '𐑻'];
    let needs_improvement = vec!['𐑨', '𐑳', '𐑤', '𐑯',
                '𐑧', '𐑓', '𐑾', '𐑽', '𐑼', '𐑻'];

    let mut state = Rc::new(RefCell::new(State::new(chars, needs_improvement)));
    let callback_state1 = Rc::clone(&state);
    let callback_state2 = Rc::clone(&state);

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::Hide)?;

    // Game loop timing
    let target_fps = 30;
    let frame_duration = Duration::from_secs_f32(1.0 / target_fps as f32);
    let mut last_frame = Instant::now();

    let mut transliterator = Shaw::new();

    transliterator.set_partial_callback(move |c| {
        (*callback_state1).borrow_mut().partial(c);
    });

    transliterator.set_complete_callback(move |c| {
         (*callback_state2).borrow_mut().push(c);
    });

    // Initial display.
    (*state).borrow_mut().update_display();

    'gameloop: loop {
        // 1. INPUT HANDLING (Non-blocking)
        while event::poll(Duration::from_secs(0))? {
            if let Event::Key(key_event) = event::read()? {
                // Ignore key release events (Windows specific behavior)
                if key_event.kind == event::KeyEventKind::Release {
                    continue;
                }
                
                match key_event.code {
                    KeyCode::Char(c) => {
                        transliterator.apply(c);
                    },
                    _ => {},
                }

                match key_event.code {
                    // Quit game
                    KeyCode::Esc => break 'gameloop,
                    
                    // Handle Ctrl+C
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        break 'gameloop
                    }

                    KeyCode::Backspace => { /* Move Up */ }

                    // Game controls
                    KeyCode::Up    => { /* Move Up */ }
                    KeyCode::Down  => { /* Move Down */ }
                    KeyCode::Left  => { /* Move Left */ }
                    KeyCode::Right => { /* Move Right */ }
                    
                    _ => {}
                }
                (*state).borrow_mut().update_display();
            }
        }

        // 2. GAME LOGIC UPDATE (Runs every frame)
        // update_game_state();

        // 3. RENDERING
        // render_screen(&mut stdout);

        // 4. FRAME RATE LIMITER
        let elapsed = last_frame.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
        last_frame = Instant::now();
    }

    // Restore terminal settings on exit
    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
