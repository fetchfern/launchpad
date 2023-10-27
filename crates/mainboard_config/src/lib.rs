use fuzzer::Fuzzable;

pub struct Command {
    name: String,
    description: String,
}

impl Fuzzable for Command {
    fn pattern(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    red: u8,
    green: u8,
    blue: u8,
}

impl Rgb {
    pub const WHITE: Rgb = Rgb::new(u8::MAX, u8::MAX, u8::MAX);
    pub const BLACK: Rgb = Rgb::new(u8::MIN, u8::MIN, u8::MIN);
    pub const ALMOST_WHITE: Rgb = Rgb::new(210, 210, 210);
    pub const ALMOST_BLACK: Rgb = Rgb::new(46, 46, 46);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub const fn red(self) -> u8 {
        self.red
    }

    pub const fn green(self) -> u8 {
        self.green
    }

    pub const fn blue(self) -> u8 {
        self.blue
    }
}
