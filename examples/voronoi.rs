extern crate voronoi;
extern crate rand;

use glam::*;
use voronoi::{voronoi, Point, make_polygons};
use rand::Rng;

static DEFAULT_WINDOW_HEIGHT: u32 = 1080;
static DEFAULT_WINDOW_WIDTH:  u32 = 1920;

struct Settings {
    lines_only: bool,
    random_count: usize
}

use ggez::*;
use ggez::{Context, GameResult};
use oorandom::Rand32;
use ggez::graphics::Rect;
use ggez::conf::WindowMode;
use std::convert::TryInto;
use core::cmp::Ordering;
use embedded_graphics::{geometry::Point as Point3, prelude::*, primitives::*};
use ggez::graphics::MeshBuilder;

struct State {
    shapes: Vec<Shape>,
    vor_polys: Vec<Vec<Point>>,
    mesh: ggez::graphics::Mesh,
}

enum Shape {
    Circle(mint::Point2<f32>, f32),
    Rectangle(ggez::graphics::Rect),
}

impl State {
    /// Load images and create meshes.
    fn new(ctx: &mut ggez::Context) -> GameResult<State> {
        let mut shapes = Vec::new();
        let countries_tot = 36.0;
        let countries_row = 6.0;
        let countries_col = 6.0;
        let max_w = 1728.0;
        let max_h = 972.0;
        let incr_w = max_w/(countries_row+1.0);
        let incr_h = max_h/(countries_col+1.0);
        let mut vor_pts = Vec::new();
        shapes.push(Shape::Rectangle(ggez::graphics::Rect::new(
            192.0,
            108.0,
            1536.0,
            864.0,
        )));
        for x in 1..countries_row as i64 + 1 {
            for y in 1..countries_col as i64 + 1 {
                let xx = (x as f32*incr_w) + (192.0/2.0);
                let yy = (y as f32*incr_h) + (108.0/2.0);
                vor_pts.push(Point::new(xx as f64, yy as f64));
                //dots.push([x, y]);
                shapes.push(Shape::Circle(
                    mint::Point2{
                        x: xx,
                        y: yy,
                    },
                    6.0,
                ));
            }
        }


        let vor_diagram = voronoi(vor_pts, DEFAULT_WINDOW_WIDTH as f64);
        let mut vor_polys = make_polygons(&vor_diagram);

        // find the dimensions of the diagram
        // we need this to set the screen coordinates to them later
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for poly in vor_polys.iter() {
            for point in poly {
                x_min = x_min.min(*point.x);
                x_max = x_max.max(*point.x);
                y_min = y_min.min(*point.y);
                y_max = y_max.max(*point.y);
            }
        }

        // let the screen-space go from (x_min, y_min) to (x_max, y_max)
        let dia_width = x_max - x_min;
        let dia_height = y_max - y_min;
        let screen_rect = ggez::graphics::Rect{x: x_min as f32, y: y_min as f32, w: dia_width as f32, h: dia_height as f32 };
        ggez::graphics::set_screen_coordinates(ctx, screen_rect).unwrap();

        let mesh = build_mesh(&mut vor_polys, ctx)?;
        let state = State { shapes: shapes, vor_polys: vor_polys, mesh };

        // event::run(ctx, event_loop3, state);
        Ok(state)
    }
}
pub fn main() -> GameResult {

    //let (mut ctx, events_loop) = ggez::ContextBuilder::new("generative_art", "awesome_person")
    let width = 1920f32;
    let height = 1080f32;
    let window_mode = WindowMode {
        width,
        height,
        resizable: true,
        ..Default::default()
    };
    let (mut ctx, event_loop2) = ContextBuilder::new("generative_art", "awesome_person")
        .window_mode(window_mode)
        .build()
        .unwrap();
    println!("{}", ggez::graphics::renderer_info(&ctx)?);
    let state = State::new(&mut ctx)?;
    event::run(ctx, event_loop2, state)
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut ggez::Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        // THIS IS THE DRAW FUNCTION. DRAW CALLS ARE DONE HERE.
        //let rect_s =  first(&self.shapes).unwrap()
        let mut rect_s = ggez::graphics::Rect::new(
            192.0,
            108.0,
            1536.0,
            864.0,
        );
        ggez::graphics::clear(ctx, ggez::graphics::Color::BLACK);
        for shape in &self.shapes {
            // Make the shape...
            let mesh = match shape {
                &Shape::Rectangle(rect) => {
                    ggez::graphics::Mesh::new_rectangle(ctx, ggez::graphics::DrawMode::fill(), rect, ggez::graphics::Color::WHITE)?
                }
                // circles appear squashed, as the aspect ratio of your screen-space doesn't match the aspect ratio of the window (1:1 vs 16:9)
                &Shape::Circle(origin, radius) => {
                    ggez::graphics::Mesh::new_circle(ctx, ggez::graphics::DrawMode::fill(), origin, radius, 0.1, ggez::graphics::Color::BLACK)?
                }
            };
            // ...and then draw it.
            ggez::graphics::draw(ctx, &mesh, ggez::graphics::DrawParam::default())?;
        }
        // in order to see anything here, we need to call draw and present
        ggez::graphics::draw(ctx, &self.mesh, ggez::graphics::DrawParam::default())?;
        ggez::graphics::present(ctx)
    }

}
fn build_mesh(vor_polys: &mut Vec<Vec<Point>>, ctx: &mut ggez::Context) -> GameResult<ggez::graphics::Mesh> {
    let mb = &mut ggez::graphics::MeshBuilder::new();
    let colors = [ggez::graphics::Color::BLUE, ggez::graphics::Color::RED, ggez::graphics::Color::YELLOW, ggez::graphics::Color::GREEN];
    let mut mint = Vec::new();
    println!("poly length {}", vor_polys.len());
    for (i, poly) in vor_polys.iter().enumerate() {
        if poly.len() > 1 {
            for j in 0..poly.len() {
                mint.push(
                    Vec2::new(poly[j].x.into_inner() as f32, poly[j].y.into_inner() as f32)
                );
            }
            // then, if your polygon points don't wrap around to the first one (which I assume)
            // you'll have to close the polygon by drawing a line from end to start
            mint.push(
                Vec2::new(poly[0].x.into_inner() as f32, poly[0].y.into_inner() as f32)
            );
        }
        else {
            println!("IN!");
        }
        // this adds a line with multiple segments to the builder, which is independent from other lines before
        mb.line(
            &mint,
            4.0,
            ggez::graphics::Color::BLUE,
        )?;
        // we need to clear the vec, because we want each polygon to be independent from previous ones
        mint.clear();
    }
    mb.build(ctx)    // CREATE the mesh
}
fn first<T>(v: &Vec<T>) -> Option<&T> {
    v.first()
}
fn sort_clockwise(a: Point3, b: Point3, center: Point3) -> Ordering {
    let d_ax = a.x - center.x;
    let d_bx = b.x - center.x;

    let cmp_ax = d_ax.cmp(&0);
    let cmp_bx = d_bx.cmp(&0);

    match (cmp_ax, cmp_bx) {
        // d_ax >= 0 && d_bx < 0
        (Ordering::Greater, Ordering::Less) | (Ordering::Equal, Ordering::Less) => {
            Ordering::Greater
        }
        // d_ax < 0 && d_bx >= 0
        (Ordering::Less, Ordering::Greater) | (Ordering::Less, Ordering::Equal) => Ordering::Less,
        // d_ax == 0 && d_bx == 0
        (Ordering::Equal, Ordering::Equal) if a.y - center.y >= 0 || b.y - center.y >= 0 => {
            a.y.cmp(&b.y)
        }
        (Ordering::Equal, Ordering::Equal) => b.y.cmp(&a.y),
        _ => {
            // Compute the cross product of vectors (center -> a) x (center -> b)
            let det = (d_ax) * (b.y - center.y) - (d_bx) * (a.y - center.y);

            match det.cmp(&0) {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => {
                    // Points a and b are on the same line from the center. Check which point is closer to
                    // the center.
                    let d1 = (d_ax) * (d_ax) + (a.y - center.y) * (a.y - center.y);
                    let d2 = (d_bx) * (d_bx) + (b.y - center.y) * (b.y - center.y);

                    d1.cmp(&d2)
                }
            }
        }
    }
}
fn demo<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}
fn random_point() -> [f64; 2] {
    [rand::thread_rng().gen_range(0., DEFAULT_WINDOW_HEIGHT as f64), rand::thread_rng().gen_range(0., DEFAULT_WINDOW_WIDTH as f64)]
}

fn random_color() -> [f32; 4] {
    [rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>(), 1.0]
}

fn random_voronoi(dots: &mut Vec<[f64;2]>, colors: &mut Vec<[f32;4]>, num: usize) {
    dots.clear();
    colors.clear();

    for _ in 0..num {
        dots.push(random_point());
        colors.push(random_color());
    }
}

