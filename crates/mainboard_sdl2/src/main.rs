use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use fuzzer::{MatchOwned, Fuzzer};
use config::Rgb;
use std::time::{Instant, Duration};
use std::thread;
use std::mem;
use std::cell::{RefMut, RefCell};
use std::rc::Rc;

fn sdl_color(rgb: Rgb) -> Color {
    Color::RGB(rgb.red(), rgb.green(), rgb.blue())
}

fn main() -> anyhow::Result<()> {
    let app = App::init()?;

    app.run()?;

    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl From<Rect> for SdlRect {
    fn from(value: Rect) -> Self {
        SdlRect::new(value.x() as _, value.y() as _, value.width(), value.height())
    }
}

impl Rect {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn offset_x(self, offset: u32) -> Rect {
        Rect::new(self.x + offset, self.y, self.w, self.h)
    }

    pub fn offset_y(self, offset: u32) -> Rect {
        Rect::new(self.x, self.y + offset, self.w, self.h)
    }

    pub fn cutoff_x(self, offset: u32) -> Rect {
        Rect::new(self.x + offset, self.y, self.w - offset, self.h)
    }

    pub fn cutoff_y(self, offset: u32) -> Rect {
        Rect::new(self.x, self.y + offset, self.w, self.h - offset)
    }

    pub fn cutoff_width(self, offset: u32) -> Rect {
        Rect::new(self.x, self.y, self.w.saturating_sub(offset), self.h)
    }

    pub fn cutoff_height(self, offset: u32) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h.saturating_sub(offset))
    }

    pub fn with_size(self, width: u32, height: u32) -> Rect {
        Rect::new(self.x, self.y, width, height)
    }

    pub fn x(self) -> u32 {
        self.x
    }

    pub fn y(self) -> u32 {
        self.y
    }
    
    pub fn x_signed(self) -> i32 {
        self.x as _
    }

    pub fn y_signed(self) -> i32 {
        self.y as _
    }

    pub fn width(self) -> u32 {
        self.w
    }

    pub fn height(self) -> u32 {
        self.h
    }

    pub fn sdl(self) -> SdlRect {
        SdlRect::new(self.x as _, self.y as _, self.w, self.h)
    }
}

pub struct Fonts {
    ctx: Sdl2TtfContext,
    default_24: Font<'static, 'static>,
    default_32: Font<'static, 'static>,
}

impl Fonts {
    pub fn init() -> Result<Self> {
        let ctx = sdl2::ttf::init().map_err(error::system_ttf)?;

        let load = |name: &str, size: u16| unsafe {
            Ok(mem::transmute::<
                Font<'_, 'static>,
                Font<'static, 'static>,
            >(ctx.load_font(name, size).map_err(error::asset_font)?))
        };

        let default_24 = load("default_font.ttf", 24)?;
        let default_32 = load("default_font.ttf", 32)?;

        Ok(Self {
            ctx,
            default_24,
            default_32,
        })
    }

    pub fn default_24(&self) -> &Font<'_, 'static> {
        unsafe { mem::transmute(&self.default_24) }
    }

    pub fn default_32(&self) -> &Font<'_, 'static> {
        unsafe { mem::transmute(&self.default_32) }
    }
}

/// Super simple spring
#[derive(Clone)]
pub struct Spring {
    target: f64,
    value: f64,
    velocity: f64,
}

impl Spring {
    pub fn new(value: f64) -> Self {
        Self {
            target: value,
            value,
            velocity: 0.0,
        }
    }

    pub fn update_target(&mut self, new_value: f64) {
        self.target = new_value;
    }

    pub fn simulate(&mut self) {
        self.velocity = self.velocity * 0.1 + (self.target - self.value) * 0.4;
        self.value += self.velocity;
    }

    pub fn value(&self) -> f64 {
        self.value }
}


pub struct Resources {
    styling: (),
    fuzzer: Fuzzer<String>,
    fonts: Fonts,
    cursor: RefCell<Spring>,
}

impl Resources {
    pub fn new() -> Result<Self> {
        Ok(Self {
            styling: (),
            fuzzer: Fuzzer::new(vec![
                                "docs".to_owned(),
                                "obsidian".to_owned(),
                                "two".to_owned(),
                                "three".to_owned(),
                                "four".to_owned(),
            ]),
            fonts: Fonts::init()?,
            cursor: RefCell::new(Spring::new(0.)),
        })
    }

    pub fn styling(&self) {
        self.styling
    }

    pub fn prompt_content(&self) -> &str {
        self.fuzzer.input()
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    pub fn cursor_spring(&self) -> RefMut<Spring> {
        self.cursor.borrow_mut()
    }
}

pub struct VirtualCanvas {
    canvas: Rc<RefCell<Canvas<Window>>>,
    area: Rect,
}

impl VirtualCanvas {
    pub fn new(canvas: Rc<RefCell<Canvas<Window>>>, area: Rect) -> Self {
        Self { canvas, area }
    }

    pub fn root(canvas: Rc<RefCell<Canvas<Window>>>) -> Self {
        let v = canvas.borrow().viewport();

        Self {
            area: Rect::new(v.x as _, v.y as _, v.width(), v.height()),
            canvas,
        }
    }

    pub fn subdivide_exact(self, height: u32) -> Option<(Self, Self)> {
        let VirtualCanvas { canvas, area } = self;

        area.height()
            .ge(&height)
            .then(|| {
                let area_subdivided = area.with_size(area.width(), height);
                let area_remaining = area.cutoff_y(height);
                (Self::new(Rc::clone(&canvas), area_subdivided), Self::new(canvas, area_remaining))
            })
    }

    pub fn subdivide_up_to(self, max_height: u32) -> (Self, Option<Self>) {
        let VirtualCanvas { canvas, area } = self;

        let area_subdivided = if area.height() >= max_height {
            area.with_size(area.width(), max_height)
        } else {
            area
        };

        let area_remaining = area.cutoff_y(max_height);

        let rem_canvas = area_remaining.height()
            .ne(&0)
            .then(|| Self::new(Rc::clone(&canvas), area_remaining));

        (Self::new(canvas, area_subdivided), rem_canvas)
    }

    pub fn fill_area(&mut self, color: Rgb, area: Rect) -> Result<()> {
        let mut canvas = self.canvas.borrow_mut();
        canvas.set_draw_color(sdl_color(color));
        canvas.fill_rect(Some(area.sdl())).map_err(error::draw)?;
        Ok(())
    }

    pub fn write_text(&mut self, content: &str, font: &Font<'_, 'static>, color: Rgb, area: Rect) -> Result<Rect> {
        let surface = font
            .render(content)
            .blended(sdl_color(color))
            .map_err(error::draw)?;

        let mut canvas = self.canvas.borrow_mut();

        let creator = canvas.texture_creator();
        let texture = surface.as_texture(&creator).map_err(error::draw)?;
        let query = texture.query();
        let (width, height) = (query.width, query.height);

        let hdelta = area.height() as i32 - height as i32;
        let dst = Rect::new(
            area.x(),
            area.y().overflowing_add_signed(hdelta / 2).0,
            width.min(area.width()),
            height.min(area.height()),
        );

        canvas.copy(
            &texture,
            Some(SdlRect::new(0, 0, area.width(), area.height())),
            Some(dst.sdl()),
        ).map_err(error::draw)?;

        Ok(dst)
    }

    pub fn area(&self) -> Rect {
        self.area
    }
}

pub trait Render {
    fn render(&self, canvas: &mut VirtualCanvas, resources: &Resources) -> Result<()>;
}

struct Prompt;

impl Render for Prompt {
    fn render(&self, canvas: &mut VirtualCanvas, resources: &Resources) -> Result<()> {
        let content = resources.prompt_content();
        let font = resources.fonts().default_32();

        const PROMPT_PAD: u32 = 10;

        canvas.fill_area(Rgb::new(32, 30, 35), canvas.area())?;
        let prompt_area = canvas.write_text(">", font, Rgb::ALMOST_WHITE, canvas.area().cutoff_x(4))?;
        let min_right = prompt_area.x() + prompt_area.width() + PROMPT_PAD;
        let mut more_right = 0;

        if !content.is_empty() {
            let text_area = canvas.area().cutoff_x(prompt_area.width() + PROMPT_PAD);
            let area = canvas.write_text(content, font, Rgb::ALMOST_WHITE, text_area)?;
            more_right = area.width();
        }

        let mut spring = resources.cursor_spring();
        spring.update_target(more_right as f64);
        spring.simulate();

        let cursor = Rect::new(min_right + spring.value() as u32 - 1, prompt_area.y(), 2, prompt_area.height());
        canvas.fill_area(Rgb::WHITE, cursor)?;

        Ok(())
    }
}

struct Choice {
    matched: MatchOwned<String>,
}

impl Render for Choice {
    fn render(&self, canvas: &mut VirtualCanvas, resources: &Resources) -> Result<()> {
        let font = resources.fonts().default_24();

        let name = &self.matched.item;

        canvas.write_text(name, font, Rgb::ALMOST_WHITE, canvas.area().cutoff_x(4))?;

        Ok(())
    }
}

pub struct App {
    context: sdl2::Sdl,
    resources: Resources,
    canvas: Rc<RefCell<Canvas<Window>>>,
}

impl App {
    pub fn init() -> Result<Self> {
        let context = sdl2::init().map_err(error::system_core)?;
        let canvas = context.video()
            .map_err(error::system_video)?
            .window("Board", 800, 600)
            .position_centered()
            .build()
            .map_err(error::window_init)?
            .into_canvas()
            .build()
            .map_err(error::window_init)?;

        Ok(App {
            context,
            resources: Resources::new()?,
            canvas: Rc::new(RefCell::new(canvas)),
        })
    }

    pub fn run(mut self) -> Result<i32> {
        let mut pump = self.context.event_pump().map_err(error::system_event_pump)?;

        'main: loop {
            let start = Instant::now();

            for ev in pump.poll_iter() {
                use Event::*;

                if matches!(ev, Quit { .. } | KeyDown { keycode: Some(Keycode::Escape), .. }) {
                    println!("Quitting");
                    break 'main Ok(0);
                }

                if let KeyDown { keycode: Some(kc), .. } = ev {
                    let name = kc.name();
                    let mut chars = name.chars().peekable();
                    let first = chars.next().expect("non-empty name");
                    let input = self.resources.fuzzer.input_mut();

                    if first.is_alphanumeric() && chars.peek().is_none() {
                        input.push(first.to_ascii_lowercase());
                    } else if matches!(kc, Keycode::Space) {
                        input.push(' ');
                    } else if matches!(kc, Keycode::Backspace) {
                        let _ = input.pop();
                    }
                }
            }

            self.render()?;

            let time = start.elapsed();
            let sixty_fps = Duration::from_secs_f64(1. / 60.);

            if sixty_fps > time {
                thread::sleep(sixty_fps - time);
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        self.canvas.borrow_mut().set_draw_color(sdl_color(Rgb::ALMOST_BLACK));
        self.canvas.borrow_mut().clear();

        let root = VirtualCanvas::root(Rc::clone(&self.canvas));

        let (mut prompt, rest) = root.subdivide_exact(64).expect("enough space for prompt");
        Prompt.render(&mut prompt, &self.resources)?;

        let input_empty = self.resources.fuzzer.input().is_empty();

        let mut rest = rest;

        let matches = self.resources
            .fuzzer
            .matches()
            .take_while(|m| m.score > 0 || input_empty)
            .map(|m| m.owned())
            .collect::<Vec<_>>();

        for m in matches {
            let choice = Choice { matched: m };
            let (mut this, maybe_new_rest) = rest.subdivide_up_to(32);

            Render::render(&choice, &mut this, &self.resources)?;

            if let Some(new_rest) = maybe_new_rest {
                println!("continue");
                rest = new_rest;
            } else {
                println!("break");
                break;
            }
        }

        self.canvas.borrow_mut().present();

        Ok(())
    }
}


pub use error::Result;

pub mod error;
