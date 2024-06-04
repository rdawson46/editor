// rgb code
#[warn(dead_code)]
type Color = (u8, u8, u8);

#[warn(dead_code)]
struct ColorConfig {
    background: Color,
    foreground: Color,
    border: Color,
    active: bool,
}

#[warn(dead_code)]
impl ColorConfig {
    // will read in from a config
    fn new() -> Self {
        ColorConfig { background: (0, 0, 0), foreground: (255, 255, 255), border: (0, 0, 0), active: true }
    }

    fn disable_style(&mut self) {
        self.active = false;
    }

    fn enable_style(&mut self) {
        self.active = true;
    }

    fn toggle_style(&mut self) {
        self.active = !self.active;
    }
}
