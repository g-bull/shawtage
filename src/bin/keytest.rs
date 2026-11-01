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

struct ShawJAFL {
    history: VecDeque<char>,
    complete_callback: Box<dyn FnMut(char) + 'static>,
    partial_callback: Box<dyn FnMut(char) + 'static>,
    is_partial: bool,
    partial_char: char,
    map : HashMap<char, char>,
}

impl ShawJAFL {

    fn set_complete_callback(&mut self, callback: impl FnMut(char) + 'static) {
        self.complete_callback = Box::new(callback);
    }

    fn set_partial_callback(&mut self, callback: impl FnMut(char) + 'static) {
        self.partial_callback = Box::new(callback);
    }

    fn new() -> Self {
        Self {
            history: VecDeque::new(),
            complete_callback: Box::new(|ev| println!("Default complete for: {ev}")),
            partial_callback: Box::new(|ev| println!("Default partial for: {ev}")),
            is_partial: false,
            partial_char: ' ',
            map: HashMap::from([
                ('q', '𐑱'), ('w', '𐑧'), ('e', '𐑰'), ('r', '𐑥'), ('t', '𐑒'), 
                ('y', '𐑐'), ('u', '𐑑'), ('i', '𐑛'), ('o', '𐑓'), ('p', ';'),
                ('a', '𐑪'), ('s', '𐑨'), ('d', '𐑩'), ('f', '𐑦'), ('g', '𐑳'),
                ('h', '𐑤'), ('j', '𐑮'), ('k', '𐑕'), ('l', '𐑯'), (';', '𐑢'),
                ('z', '𐑲'), ('x', '𐑴'), ('c', '𐑞'), ('v', '𐑟'), ('b', '𐑣'),
                ('n', '𐑝'), ('m', '𐑚'),
                ('Q', '𐑬'), ('W', '𐑹'), ('E', '𐑸'), ('R', '𐑿'), ('T', '𐑜'),
                ('Y', '\u{2e30}'),   // \u{2060}{2060}' Ring Point + WORD JOINER + WORD JOINER
                ('U', '𐑗'), ('I', '𐑡'),
                ('O', '\u{2013}'),	// EN DASH
                ('P', ':'),
                ('A', '𐑷'), ('S', '𐑭'), ('D', '𐑩'), ('F', '𐑵'), ('G', '𐑫'),
                ('H', '\u{00b7}'),	// MIDDLE DOT
                ('J', '𐑮'), ('K', '𐑖'), ('L', '𐑙'), (':', '𐑘'), ('Z', '𐑶'),
                ('X', '\u{fe00}'),	// VARIATION SELECTOR
                ('C', '𐑔'), ('V', '𐑠'),
                ('B', '\u{2039}'),	// SINGLE LEFT-POINTING ANGLE QUOTATION MARK
                ('N', '\u{203a}'),	// SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
                ('M', '\u{2014}'),	// EM DASH
            ]),
        }
    }

    fn latin2shaw (&self, latin : char) -> char {
        match self.map.get(&latin) {
            Some(shaw) => return *shaw,
            None => latin,
        }
    }

    fn complete(&mut self) {
        if self.is_partial {
            self.is_partial = false;
            (self.complete_callback)(self.partial_char);
        }

    }

    fn make_partial(&mut self, c: char) {
        self.partial_char = c;
        self.is_partial = true;
        (self.partial_callback)(self.partial_char);
    }

    fn apply (&mut self, latin : char) {
        let shaw = self.latin2shaw(latin);
        if (latin.is_ascii_uppercase() 
            && !['𐑷' , '𐑭', '𐑘', '𐑵', '‹', '›'].contains(&shaw))
         || latin == shaw {
            // complete any pending char
            // complete shaw
            self.complete();
            self.make_partial(shaw);
            self.complete();
        }
        else {
            match shaw {
                '𐑦' | '𐑱' | '𐑳' | '𐑘' | '𐑷' | '𐑭' => {
                    // start of 𐑾, 𐑽, 𐑺, 𐑻, 𐑹, 𐑸 
                    self.complete();
                    self.make_partial(shaw);
                },
                '‹' => {
                    // start or finish of «  
                    if self.is_partial && self.partial_char == '‹' {
                        self.make_partial('«');   
                        self.complete();
                    }    
                    else {
                        self.complete();
                        self.make_partial(shaw);                                   
                    }
                },
                '›' => {
                    // start or finish of »  
                    if self.is_partial && self.partial_char == '›' {
                        self.make_partial('»');   
                        self.complete();
                    }    
                    else {
                        self.complete();
                        self.make_partial(shaw);                                   
                    }
                },
                '𐑩' => {
                    if self.is_partial && self.partial_char == '𐑦' {
                        self.make_partial('𐑾');   
                    }    
                    else {
                        self.complete();
                        self.make_partial(shaw);                                   
                    }
                },
                '𐑵' => {
                    if self.is_partial && self.partial_char == '𐑘' {
                        self.make_partial('𐑿');   
                        self.complete();
                    }
                    else {
                        self.complete();
                        self.make_partial(shaw);
                        self.complete();
                    }                        
                },
                '‹' => {
                    if self.is_partial && self.partial_char == '‹' {
                        self.make_partial('«');   
                        self.complete();
                    }
                    else {
                        self.complete();
                        self.make_partial(shaw);
                        self.complete();
                    }                                             
                },
                '›' => {
                    if self.is_partial && self.partial_char == '›' {
                        self.make_partial('»');   
                        self.complete();
                    }
                    else {
                        self.complete();
                        self.make_partial(shaw);
                        self.complete();
                    }                                             
                },
                '𐑮' => {
                    if self.is_partial {
                        match self.partial_char {
                            '𐑦' => {
                                self.complete();
                                self.make_partial(shaw);
                                self.complete();
                            },
                            '𐑾' => {
                                self.make_partial('𐑽');
                                self.complete();
                            },
                            '𐑱' => {
                                self.make_partial('𐑺');
                                self.complete();
                            },
                            '𐑳' => {
                                self.make_partial('𐑻');
                                self.complete();
                            },
                            '𐑷' => {
                                self.make_partial('𐑹');
                                self.complete();
                            },
                            '𐑭' => {
                                self.make_partial('𐑸');
                                self.complete();
                            },
                            '𐑩' => {
                                self.make_partial('𐑼');
                                self.complete();
                            },
                            _ => unreachable!("Unreachable statement hit!"),
                        }
                    }
                    else {
                        self.make_partial(shaw);
                        self.complete();
                    }    
                },

                _ => {
                    self.complete();
                    self.make_partial(shaw);
                    self.complete();
                },
            }
            
        }

    }

}




fn main() -> io::Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::Hide)?;

    // Game loop timing
    let target_fps = 30;
    let frame_duration = Duration::from_secs_f32(1.0 / target_fps as f32);
    let mut last_frame = Instant::now();
    let mut y_pos = 5;
    let mut x_pos = 10;

    let mut state = ShawJAFL::new();

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
                        state.apply(c);
                        /*
                        let mut stdout = io::stdout();

                        // Move cursor to column 10, row 5 (0-indexed: col, row) and print 'X'
                        execute!(
                            stdout,
                            MoveTo(x_pos, y_pos),
                            Print(state.latin2shaw(c))
                        )?;

                        x_pos += 1;

                        // Flush to make sure it renders immediately
                        stdout.flush()?;

                        //Ok(())
                        */

                    },
                    //println!("Character: {} => {}", c, latin2shaw(c)),
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
