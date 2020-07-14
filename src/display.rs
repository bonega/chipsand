use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{pixels};
use std::error::Error;

const SCALE_FACTOR: usize = 4;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const WHITE: [u8; 3] = [224, 248, 208];
const LIGHT_GRAY: [u8; 3] = [136, 192, 112];
const DARK_GRAY: [u8; 3] = [52, 104, 86];
const BLACK: [u8; 3] = [8, 24, 32];

pub struct Display {
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys
            .window(
                "Chipsand",
                (SCREEN_WIDTH * SCALE_FACTOR) as u32,
                (SCREEN_HEIGHT * SCALE_FACTOR) as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_scale(4 as f32, 4 as f32).unwrap();
        canvas.clear();
        canvas.present();

        Display { canvas }
    }

    pub fn draw(&mut self, pixels: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let col = match col {
                    0 => WHITE,
                    1 => LIGHT_GRAY,
                    2 => DARK_GRAY,
                    3 => BLACK,
                    _ => panic!(),
                };
                self.canvas
                    .set_draw_color(pixels::Color::RGB(col[0], col[1], col[2]));
                self.canvas
                    .draw_point(sdl2::rect::Point::new(x as i32, y as i32)).unwrap();
            }
        }
        self.canvas.present();
    }

    pub fn save_screenshot(&mut self) -> Result<(), Box<dyn Error>> {
        let (scale_x, scale_y) = self.canvas.scale();
        self.canvas.set_scale(1 as f32, 1 as f32)?;
        {
            let canvas = &self.canvas;
            let pixel_format = canvas.default_pixel_format();
            let mut pixels = canvas.read_pixels(canvas.viewport(), pixel_format).unwrap();
            let (width, height) = canvas.output_size().unwrap();
            let pitch = pixel_format.byte_size_of_pixels(width as usize);
            let surf = sdl2::surface::Surface::from_data(
                pixels.as_mut_slice(),
                width,
                height,
                pitch as u32,
                pixel_format,
            )
            .unwrap();
            surf.save_bmp("test.bmp")?;
        }
        self.canvas.set_scale(scale_x, scale_y)?;
        Ok(())
    }
}
