use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use fuzzer::Fuzzer;
use crate::error;
use crate::style::{Rgb, Style};
use crate::assets::AssetSupervisor;

struct State {
    valign: i32,
    width: u32,
    height: u32
}

impl State {
    pub fn new(renderer: &Renderer) -> Self {
        Self {
            valign: 0,
            width: renderer.width,
            height: renderer.height,
        }
    }

    pub fn new_section(&mut self, height: u32) -> Rect {
        let start = self.valign;
        let delim = start + height as i32;

        self.valign = delim;

        Rect::new(0, start, self.width, height)
    }
}

/// Super simple spring
struct ScalarSpring {
    target: f64,
    value: f64,
    velocity: f64,
}

impl ScalarSpring {
    pub fn at(value: f64) -> Self {
        ScalarSpring {
            target: value,
            value,
            velocity: 0.0,
        }
    }

    pub fn update_target(&mut self, new_value: f64) {
        self.target = new_value;
    }

    pub fn next_value(&mut self) {
        self.velocity = self.velocity * 0.1 + (self.target - self.value) * 0.4;
        self.value += self.velocity;
    }
}

pub struct Renderer {
    canvas: Canvas<Window>,
    style: Style,
    width: u32,
    height: u32,
    assets: AssetSupervisor,
    cursor_spring: ScalarSpring,
}

impl Renderer {
    pub fn init(sdl: &sdl2::Sdl, style: Style, width: u32, height: u32) -> crate::Result<Self> {
        let video = sdl.video().map_err(error::system_video)?;

        let window = video.window("mainboard-demo", width, height)
            .position_centered()
            .build()
            .map_err(error::window_init)?;

        let canvas = window.into_canvas().build().map_err(error::window_init)?;

        Ok(Self {
            canvas,
            style,
            width,
            height,
            assets: AssetSupervisor::init()?,
            cursor_spring: ScalarSpring::at(0.),
        })
    }

    pub fn render(&mut self, fuzzer: &Fuzzer, slim: bool) -> crate::Result<()> {
        let mut state = State::new(self);

        self.canvas.set_draw_color(self.style.background);
        self.canvas.clear();

        if !slim { 
            render_prompt(self, &mut state, fuzzer)?;
            render_results(self, &mut state, fuzzer)?;
        }

        self.canvas.present();

        Ok(())
    }
}

fn render_results(renderer: &mut Renderer, state: &mut State, fuzzer: &Fuzzer) -> crate::Result<()> {
    let canvas = &mut renderer.canvas;

    for item in fuzzer.matches() {
        let section = state.new_section(32);

        let _ = renderer.assets.copy_font_onto(
            canvas,
            &item.command().unwrap().name,
            renderer.assets.font_default_16(),
            section,
            |p| p.blended(Rgb::WHITE),
        );
    }

    Ok(())
}

fn render_prompt(renderer: &mut Renderer, state: &mut State, fuzzer: &Fuzzer) -> crate::Result<()> {
    let prompt = state.new_section(56);
    let content = fuzzer.input();

    // background
    let canvas = &mut renderer.canvas;
    canvas.set_draw_color(renderer.style.accent_background);
    canvas.fill_rect(prompt).map_err(error::draw)?;

    const TEXT_OFFSET: i32 = 6;

    // draw placeholder text
    let mut p_rect = renderer.assets.copy_font_onto(
        canvas,
        ">",
        renderer.assets.font_default_32(),
        {
            // mfw mutable Copy structs
            let mut font_rect = prompt;
            font_rect.offset(TEXT_OFFSET, 0);
            font_rect
        },
        |p| p.blended(Rgb::ALMOST_WHITE),
    )?;

    p_rect.set_x(p_rect.x() + TEXT_OFFSET * 2);

    // draw the content text
    let text_dst = if !content.is_empty() {
        renderer.assets.copy_font_onto(
            canvas,
            content,
            renderer.assets.font_default_32(),
            {
                let corner = p_rect.top_right();
                Rect::new(
                    corner.x,
                    prompt.y,
                    prompt.width() - corner.y as u32 + prompt.y() as u32,
                    prompt.height(),
                )
            },
            |p| p.blended(Rgb::ALMOST_WHITE),
        )?
    } else {
        let corner = p_rect.top_right();
        Rect::new(corner.x, corner.y, 0, p_rect.height())
    };

    // cursor
    canvas.set_draw_color(Rgb::WHITE);
    let cursor_height = (text_dst.height() as f64 / 1.3) as u32;
    let cursor_x_target = text_dst.top_right().x + TEXT_OFFSET / 2;

    let spring = &mut renderer.cursor_spring;
    spring.update_target(cursor_x_target as _);
    spring.next_value();

    let value = spring.value as i32;

    let cursor = Rect::new(
        value,
        text_dst.y + (text_dst.height() - cursor_height) as i32 / 2,
        3,
        cursor_height,
    );
    canvas.fill_rect(cursor).map_err(error::draw)?;


    Ok(())
}

fn render_choices(renderer: &mut Renderer, state: &mut State, fuzzer: &Fuzzer) -> crate::Result<()> {
    
}
