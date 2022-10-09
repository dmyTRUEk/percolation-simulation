// #![feature(generic_const_exprs)]
// #![recursion_limit = "65536"]

/// Percolation Simulation

use iced::{
    Alignment,
    Application,
    Canvas,
    Color,
    Column,
    Command,
    Element,
    Length,
    Point,
    Rectangle,
    Row,
    Settings,
    Size,
    Slider,
    Text,
    canvas,
    executor,
    slider,
};

// mod simple_rng;
// use simple_rng::SimpleRng;
use rand::{thread_rng, Rng, rngs::ThreadRng};



// const GRAPH_W: usize = 5;
// const GRAPH_H: usize = 3;
const GRAPH_W: usize = 360;
const GRAPH_H: usize = 180;
// const GRAPH_W: usize = 1920;
// const GRAPH_H: usize = 990;
const GRAPH_WH: usize = GRAPH_W * GRAPH_H;

const INITIAL_PARAMETER: f32 = 0.5;



pub fn main() -> iced::Result {
    PercolationState::run(
        Settings {
            antialiasing: false, // otherwise it dont work for me
            exit_on_close_request: true,
            text_multithreading: true,
            try_opengles_first: true,
            ..Settings::default()
        }
    )
}



#[derive(Debug, Clone, Copy)]
enum Message {
    SliderChanged(f32),
}

#[derive(Default, Debug)]
struct PercolationState {
    graph: PercolationGraph<GRAPH_WH>, // const
    parameter: f32,
    slider_state: slider::State,
}

impl Application for PercolationState {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                parameter: INITIAL_PARAMETER,
                ..Self::default()
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("Percolation Simulation")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Self::Message::SliderChanged(new_value) => {
                self.parameter = new_value;
                self.graph.parameter = new_value;
                self.graph.redraw();
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center)
            .push(
                Canvas::new(&self.graph)
                    .width(Length::Units(1920))
                    .height(Length::Units(990))
            )
            .push(Row::new()
                .padding(10)
                .push(
                    Slider::new(
                        &mut self.slider_state,
                        0.0..=1.0,
                        self.parameter,
                        Self::Message::SliderChanged
                    )
                    .step(0.00001)
                )
                .push(
                    Text::new(format!("{:.4}", self.parameter))
                )
            )
            .into()
    }
}

#[derive(Debug)]
struct PercolationGraph<const N: usize> {
    // edge: [f32; N],
    edge: Vec<f32>,
    parameter: f32,
    cache: canvas::Cache,
}

impl<const N: usize> PercolationGraph<N> {
    pub fn new() -> Self {
        let mut pg = Self {
            edge: vec![0.0; N],
            parameter: INITIAL_PARAMETER,
            cache: canvas::Cache::default()
        };
        pg.fill_random();
        pg
    }

    pub fn redraw(&mut self) {
        self.cache.clear();
    }

    pub fn fill_random(&mut self) {
        // let mut rng: SimpleRng = SimpleRng::new(42);
        let mut rng: ThreadRng = thread_rng();
        for i in 0..N {
            self.edge[i] = rng.gen_range(0.0..1.0);
        }
    }

    // // TODO?: rewrite to trait `Index`
    // pub fn get_edge_at(&self, w: usize, h: usize) -> f32 {
    //     self.edge[h*GRAPH_W + w]
    // }

    // TODO
    // pub fn get_color_at(w: usize, h: usize) -> Color {}
    // pub fn draw_cluster(_index: usize) {}
}

impl<const N: usize> canvas::Program<Message> for &PercolationGraph<N> {
    fn update(
        &mut self,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: canvas::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let _cursor_position =
            if let Some(position) = cursor.position_in(&bounds) {
                position
            } else {
                return (canvas::event::Status::Ignored, None);
            };

        match event {
            canvas::Event::Mouse(_mouse_event) => {
                // let message = match mouse_event {
                //     iced::mouse::Event::ButtonPressed(
                //         iced::mouse::Button::Left,
                //     ) => Some(Message::PointAdded(cursor_position)),
                //     iced::mouse::Event::ButtonPressed(
                //         iced::mouse::Button::Right,
                //     ) => Some(Message::PointRemoved),
                //     _ => None,
                // };
                // (event::Status::Captured, message)
                (canvas::event::Status::Ignored, None)
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _cursor: canvas::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geom: canvas::Geometry = self.cache.draw(bounds.size(), |frame| {

            #[derive(Debug, Clone, Copy, PartialEq)]
            struct MyColor { r: u8, g: u8, b: u8 }
            impl MyColor {
                const fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
                fn random(rng: &mut ThreadRng) -> Self {
                    Self::new(
                        rng.gen_range(0..255) as u8,
                        rng.gen_range(0..255) as u8,
                        rng.gen_range(0..255) as u8,
                    )
                }
            }
            const WHITE: MyColor = MyColor::new(255, 255, 255);
            #[derive(Debug, Clone, Copy)]
            struct MyPoint { w: usize, h: usize }
            impl MyPoint {
                const fn new(w: usize, h: usize) -> Self { Self { w, h } }
                const fn left(&self)  -> Self { Self::new(self.w-1, self.h) }
                const fn right(&self) -> Self { Self::new(self.w+1, self.h) }
                const fn up(&self)    -> Self { Self::new(self.w, self.h+1) }
                const fn down(&self)  -> Self { Self::new(self.w, self.h-1) }
            }

            // TODO: try to convert in plain array/vec
            type Pixels = [[MyColor; GRAPH_H]; GRAPH_W];

            fn fill_cluster(pixels: &mut Pixels, start_point: MyPoint, color: MyColor, p: f64, rng: &mut ThreadRng) {
                let (w, h) = (start_point.w, start_point.h);
                if pixels[w][h] != WHITE { return; }
                pixels[w][h] = color;
                if w > 0 && pixels[w-1][h] == WHITE && rng.gen_bool(p) {
                    fill_cluster(pixels, start_point.left(), color, p, rng);
                }
                if w < GRAPH_W - 1 && pixels[w+1][h] == WHITE && rng.gen_bool(p) {
                    fill_cluster(pixels, start_point.right(), color, p, rng);
                }
                if h > 0 && pixels[w][h-1] == WHITE && rng.gen_bool(p) {
                    fill_cluster(pixels, start_point.down(), color, p, rng);
                }
                if h < GRAPH_H - 1 && pixels[w][h+1] == WHITE && rng.gen_bool(p) {
                    fill_cluster(pixels, start_point.up(), color, p, rng);
                }
            }

            // let mut rng: SimpleRng = SimpleRng::new(42);
            let mut rng: ThreadRng = thread_rng();
            let mut pixel: Pixels = [[WHITE; GRAPH_H]; GRAPH_W];

            let scale_w: f32 = frame.size().width  / GRAPH_W as f32;
            let scale_h: f32 = frame.size().height / GRAPH_H as f32;
            let p: f64 = self.parameter as f64;
            for h in 0..GRAPH_H {
                for w in 0..GRAPH_W {
                    fill_cluster(&mut pixel, MyPoint::new(w, h), MyColor::random(&mut rng), p, &mut rng);

                    // at this time pixel[w][h] WILL be ready

                    let path = canvas::Path::rectangle(
                        Point::new(w as f32 * scale_w, h as f32 * scale_h),
                        Size::new(scale_w, scale_h)
                    );
                    let color: MyColor = pixel[w][h];
                    frame.fill(&path, Color::from_rgb8(color.r, color.g, color.b));
                }
            }
        });
        vec![geom]
    }
}

impl<const N: usize> Default for PercolationGraph<N> {
    fn default() -> Self {
        PercolationGraph::new()
    }
}

