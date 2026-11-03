use std::cell::RefCell;
use std::rc::Rc;

use shawtage::data::words::Words;

use crossterm::{
    cursor::MoveTo,
    //Result,
};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, terminal,
};
use std::io::{self, stdout};
use std::time::{Duration, Instant};

use std::collections::{HashSet};

use std::convert::TryFrom;

mod jafl;
use jafl::Shaw;

struct Pos {
    row: u16,
    col: u16,
}
struct State {
    line: String,
    next_line: String,
    typed: Vec<char>,
    errors: Vec<char>,
    partial_char: Option<char>,
    line_pos: Pos,
    next_line_pos: Pos,
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
            next_line: words.line(CHARS_PER_LINE, &least_accurate),
            typed: Vec::new(),
            errors: Vec::new(),
            partial_char: None,
            line_pos: Pos{row: 3, col: 10},
            next_line_pos: Pos{row: 6, col: 10},
            typed_pos: Pos{row: 3, col: 10},
            partial_pos: Pos{row: 4, col: 10},
        }
    }

    fn _display(&self, pos: &Pos, s: &String, color: Color) {
        let mut stdout = io::stdout();

        let _result = execute!(
            stdout,
            MoveTo(pos.col, pos.row),
            SetForegroundColor(color),
            Print(s),
            ResetColor
        );
    }

    fn update_display(&self)
    {
        const COLOR_OK: Color      = Color::DarkGreen;   
        const COLOR_ERROR: Color   = Color::Red;  
        const COLOR_PARTIAL: Color = Color::DarkYellow;
        const COLOR_LINE: Color  = Color::White;
        const COLOR_NEXT: Color  = Color::Grey;

        let clear_line = " ".repeat(52);
        self._display(&self.line_pos, &clear_line, COLOR_LINE);
        self._display(&self.next_line_pos, &clear_line, COLOR_NEXT);
        self._display(&self.typed_pos, &clear_line, COLOR_OK);
        self._display(&self.partial_pos, &clear_line, COLOR_PARTIAL);


        self._display(&self.line_pos, &self.line, COLOR_LINE);
        self._display(&self.next_line_pos, &self.next_line, COLOR_NEXT);

        let typed_s: String = self.typed.clone().into_iter().collect();
        self._display(&self.typed_pos, &typed_s, COLOR_OK);

        let error_pos = Pos {row: self.typed_pos.row, 
                            col: self.typed_pos.col
                                + self.typed.len() as u16};

        let error_s: String = self.errors.clone().into_iter().collect();
        self._display(&error_pos, &error_s, COLOR_ERROR);

        let pos = Pos {row: self.partial_pos.row, 
                            col: self.partial_pos.col 
                                + self.typed.len() as u16
                                + self.errors.len() as u16};

        let partial_s: String = self.partial_char.into_iter().collect();
        self._display(&pos, &partial_s, COLOR_PARTIAL);
    }

    fn partial(&mut self, c: char) {
        self.partial_char = Some(c);
    }

    fn push(&mut self, c: char) {
        self.partial_char = None;

        // If c doesn't match the equivalent position in the target,
        // or there are already errors, push c to errors, 
        // other wise push it to typed.
        let target = self.line.chars().nth(self.typed.len());
        if self.errors.len() > 0
            || target.is_none() /* typed past end of line */
            || target.unwrap() != c
        {
            self.errors.push(c);
        }
        else {
            self.typed.push(c);
        }
    }

    fn pop(&mut self) -> Option<char> {
        if self.partial_char.is_none() {
            if self.errors.len() > 0 {
                return self.errors.pop();
            }
            else {
                return self.typed.pop();
            }
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

                    KeyCode::Backspace => { 
                        (*state).borrow_mut().pop();
                        transliterator.clear();
                    }

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
