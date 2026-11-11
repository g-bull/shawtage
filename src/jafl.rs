use std::collections::{HashMap};

pub enum KeyEvent {
    Complete(char),
    Partial(char),
}

pub struct Shaw {
    key_event_callback: Box<dyn FnMut(KeyEvent) + 'static>,
    partial_char: Option<char>,
    map : HashMap<char, char>,
}

impl Shaw {

    pub fn set_callback(&mut self, callback: impl FnMut(KeyEvent) + 'static) {
        self.key_event_callback = Box::new(callback);
    }

    pub fn key_map(self) -> HashMap<char, char> {
        self.map
    }

    pub fn new() -> Self {
        Self {
            key_event_callback: Box::new(|_| println!("Default key_event_callback.")),
            partial_char: None,
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

    pub fn clear(&mut self) {
        self.partial_char = None;        
    }

    fn complete(&mut self) {
        if !self.partial_char.is_none() {
            (self.key_event_callback)(KeyEvent::Complete(self.partial_char.unwrap()));
            self.partial_char = None;
        }
    }

    fn make_partial(&mut self, c: char) {
        self.partial_char = Some(c);
        (self.key_event_callback)(KeyEvent::Partial(c));
    }

    pub fn apply (&mut self, latin : char) {
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
                    if self.partial_char == Some('‹') {
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
                    if self.partial_char == Some('›') {
                        self.make_partial('»');   
                        self.complete();
                    }    
                    else {
                        self.complete();
                        self.make_partial(shaw);                                   
                    }
                },
                '𐑩' => {
                    if self.partial_char == Some('𐑦') {
                        self.make_partial('𐑾');   
                    }    
                    else {
                        self.complete();
                        self.make_partial(shaw);                                   
                    }
                },
                '𐑵' => {
                    if self.partial_char == Some('𐑘') {
                        self.make_partial('𐑿');   
                        self.complete();
                    }
                    else {
                        self.complete();
                        self.make_partial(shaw);
                        self.complete();
                    }                        
                },
                '𐑮' => {
                    match self.partial_char {
                        Some('𐑦') => {
                            self.complete();
                            self.make_partial(shaw);
                            self.complete();
                        },
                        Some('𐑾') => {
                            self.make_partial('𐑽');
                            self.complete();
                        },
                        Some('𐑱') => {
                            self.make_partial('𐑺');
                            self.complete();
                        },
                        Some('𐑳') => {
                            self.make_partial('𐑻');
                            self.complete();
                        },
                        Some('𐑷') => {
                            self.make_partial('𐑹');
                            self.complete();
                        },
                        Some('𐑭') => {
                            self.make_partial('𐑸');
                            self.complete();
                        },
                        Some('𐑩') => {
                            self.make_partial('𐑼');
                            self.complete();
                        },
                        None => {
                            self.make_partial(shaw);
                            self.complete();
                        }
                        _ => {
                            println!("\npartial: '{}', shaw: '{}'", self.partial_char.unwrap_or('*'), shaw);
                            unreachable!("Unreachable statement hit!");
                        }
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


