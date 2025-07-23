use itertools::Itertools;
use macroquad::prelude::*;

fn lorenz(p: &macroquad::math::Vec3, sigma: f32, beta: f32, rho: f32) -> macroquad::math::Vec3 {
    let x = sigma * (p.y - p.x);
    let y = p.x * (rho - p.z) - p.y;
    let z = p.x * p.y - beta * p.z;
    macroquad::math::Vec3 { x, y, z }
}

fn lorenz_integrate(
    p: &macroquad::math::Vec3,
    sigma: f32,
    beta: f32,
    rho: f32,
    dt: f32,
) -> macroquad::math::Vec3 {
    let d = lorenz(p, sigma, beta, rho);
    *p + d * dt
}

struct State {
    sigma: f32,
    beta: f32,
    rho: f32,
    dt: f32,
    tail: f32,
    speed: f32,
    start: macroquad::math::Vec3,
    points: std::collections::VecDeque<macroquad::math::Vec3>,
}

impl State {
    fn new() -> Self {
        let start = macroquad::math::vec3(0.0, 1.0, 1.05);
        Self {
            sigma: 10.0,
            beta: 8.0 / 3.0,
            rho: 28.0,
            dt: 0.005,
            tail: 5_000.0,
            speed: 10.0,
            start,
            points: std::collections::VecDeque::from([start]),
        }
    }

    fn step(&mut self) {
        let max_len = self.tail as usize;
        for _ in 0..self.speed as usize {
            self.points.push_back(lorenz_integrate(
                self.points.back().unwrap(),
                self.sigma,
                self.beta,
                self.rho,
                self.dt,
            ));
            while self.points.len() > max_len {
                self.points.pop_front();
            }
        }
    }

    fn draw(&self) {
        macroquad::models::draw_grid(
            12,
            10.,
            macroquad::color::DARKGRAY,
            macroquad::color::DARKGRAY,
        );
        self.points
            .iter()
            .tuple_windows()
            .enumerate()
            .for_each(|(i, (start, end))| {
                let d = (*end - *start).length().clamp(0.0, 2.0) / 2.0;
                macroquad::models::draw_line_3d(
                    *start,
                    *end,
                    macroquad::color::hsl_to_rgb(1.0 - d, 1.0, 0.5)
                        .with_alpha(i as f32 / self.points.len() as f32),
                );
            });
    }
}

struct OrbitCamera {
    distance: f32,
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
    pan_sensitivity: f32,
    target: macroquad::math::Vec3,
    last_left_mouse: Option<macroquad::math::Vec2>,
    last_right_mouse: Option<macroquad::math::Vec2>,
}

impl OrbitCamera {
    fn new() -> Self {
        Self {
            distance: 100.0,
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.005,
            pan_sensitivity: 0.001,
            target: macroquad::math::vec3(0.0, 0.0, 0.0),
            last_left_mouse: None,
            last_right_mouse: None,
        }
    }

    fn update(&mut self) {
        if macroquad::input::is_mouse_button_down(macroquad::input::MouseButton::Left) {
            let mouse = macroquad::input::mouse_position().into();
            if let Some(last) = self.last_left_mouse {
                let delta: macroquad::math::Vec2 = mouse - last;
                self.yaw -= delta.x * self.sensitivity;
                self.pitch += delta.y * self.sensitivity;
                self.pitch = self.pitch.clamp(
                    -std::f32::consts::FRAC_PI_2 + 0.1,
                    std::f32::consts::FRAC_PI_2 - 0.1,
                );
            }
            self.last_left_mouse = Some(mouse);
        } else {
            self.last_left_mouse = None;
        }

        if macroquad::input::is_mouse_button_down(macroquad::input::MouseButton::Right) {
            let mouse = macroquad::input::mouse_position().into();
            if let Some(last) = self.last_right_mouse {
                let forward = (self.target - self.get_position()).normalize();
                let right = forward.cross(vec3(0.0, 1.0, 0.0)).normalize();
                let up = right.cross(forward).normalize();
                let delta: macroquad::math::Vec2 = mouse - last;
                self.target -= right * delta.x * self.pan_sensitivity * self.distance;
                self.target += up * delta.y * self.pan_sensitivity * self.distance;
            }
            self.last_right_mouse = Some(mouse);
        } else {
            self.last_right_mouse = None;
        }

        self.distance -= macroquad::input::mouse_wheel().1 * 5.0;
        self.distance = self.distance.clamp(1.0, 200.0);
    }

    fn get_position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + vec3(x, y, z)
    }

    fn get_camera(&self) -> macroquad::camera::Camera3D {
        macroquad::camera::Camera3D {
            position: self.get_position(),
            up: macroquad::math::vec3(0.0, 1.0, 0.0),
            target: self.target,
            ..Default::default()
        }
    }
}

fn draw_ui(state: &mut State) {
    macroquad::ui::root_ui().window(
        1,
        macroquad::math::vec2(10.0, 10.0),
        macroquad::math::vec2(250.0, 155.0),
        |ui| {
            ui.slider(2, "sigma", -20.0..20.0, &mut state.sigma);
            ui.slider(3, "beta", -20.0..20.0, &mut state.beta);
            ui.slider(4, "rho", -20.0..40.0, &mut state.rho);
            ui.slider(5, "tail", 10.0..10_000.0, &mut state.tail);
            ui.slider(6, "speed", 1.0..20.0, &mut state.speed);
            if ui.button(None, "reset params") {
                state.sigma = 10.0;
                state.beta = 8.0 / 3.0;
                state.rho = 28.0;
            }
            if ui.button(None, "reset position") {
                state.points.clear();
                state.points.push_back(state.start);
            }
        },
    );
}

#[macroquad::main("Lorenz attractor")]
async fn main() {
    let mut state = State::new();
    let mut camera = OrbitCamera::new();

    loop {
        macroquad::window::clear_background(macroquad::color::BLACK);

        draw_ui(&mut state);
        if !macroquad::ui::root_ui().is_mouse_over(macroquad::input::mouse_position().into()) {
            camera.update();
        }
        macroquad::camera::set_camera(&camera.get_camera());
        state.step();
        state.draw();

        macroquad::window::next_frame().await
    }
}
