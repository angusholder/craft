use std::io::prelude::*;
use std::fs::{ File, OpenOptions };
use std::path::PathBuf;

use fnv::FnvHashMap;
use glium;
use glium::{ Display, Frame, Surface };
use glium::glutin;
use glium::glutin::{ Event, WindowEvent, KeyboardInput, VirtualKeyCode, CursorState, AxisId };
use glium::glutin::{ ElementState, MouseScrollDelta, MouseButton, MouseCursor, DeviceEvent };
use glium::index::PrimitiveType;
use imgui::{ ImStr, ImGui, ImGuiKey };
use imgui_glium_renderer::Renderer as ImGuiRenderer;
use toml;

use block::{ BlockType, Block };
use chunk_manager::ChunkManager;
use line_renderer::LineRenderer;
use math::*;
use player::Player;
use utils::*;


pub struct Craft {
    aspect_ratio: f32,
    width: u32,
    height: u32,
    mouse_buttons: [bool; 5],
    mouse_grabbed: bool,
    show_debug: bool,
    keys: [bool; VirtualKeyCode::Yen as usize],

    line_renderer: LineRenderer,
    chunk_manager: ChunkManager,
    tick: u64,
    player: Player,
}

fn init_imgui_keymap(imgui: &mut ImGui) {
    imgui.set_imgui_key(ImGuiKey::Tab, VirtualKeyCode::Tab as u8);
    imgui.set_imgui_key(ImGuiKey::LeftArrow, VirtualKeyCode::Left as u8);
    imgui.set_imgui_key(ImGuiKey::RightArrow, VirtualKeyCode::Right as u8);
    imgui.set_imgui_key(ImGuiKey::UpArrow, VirtualKeyCode::Up as u8);
    imgui.set_imgui_key(ImGuiKey::DownArrow, VirtualKeyCode::Down as u8);
    imgui.set_imgui_key(ImGuiKey::PageUp, VirtualKeyCode::PageUp as u8);
    imgui.set_imgui_key(ImGuiKey::PageDown, VirtualKeyCode::PageDown as u8);
    imgui.set_imgui_key(ImGuiKey::Home, VirtualKeyCode::Home as u8);
    imgui.set_imgui_key(ImGuiKey::End, VirtualKeyCode::End as u8);
    imgui.set_imgui_key(ImGuiKey::Delete, VirtualKeyCode::Delete as u8);
    imgui.set_imgui_key(ImGuiKey::Backspace, VirtualKeyCode::Back as u8);
    imgui.set_imgui_key(ImGuiKey::Enter, VirtualKeyCode::Return as u8);
    imgui.set_imgui_key(ImGuiKey::Escape, VirtualKeyCode::Escape as u8);
    imgui.set_imgui_key(ImGuiKey::A, VirtualKeyCode::A as u8);
    imgui.set_imgui_key(ImGuiKey::C, VirtualKeyCode::C as u8);
    imgui.set_imgui_key(ImGuiKey::V, VirtualKeyCode::V as u8);
    imgui.set_imgui_key(ImGuiKey::X, VirtualKeyCode::X as u8);
    imgui.set_imgui_key(ImGuiKey::Y, VirtualKeyCode::Y as u8);
    imgui.set_imgui_key(ImGuiKey::Z, VirtualKeyCode::Z as u8);
}

fn load_settings() {
    if let Ok(mut file) = File::open("settings.toml") {
        let mut string = String::new();
        file.read_to_string(&mut string);
        match toml::de::from_str(&string) {
            Ok(settings) => {
                unsafe {
                    SETTINGS_MUT = settings;
                }
                info!("Loaded settings successfully.");
            }
            Err(e) => {
                warn!("Failed to parse settings: {}.", e);
            }
        }
    } else {
        info!("No settings found, continuing with defaults.");
    }
}

fn store_settings() {
    let settings_string = unsafe {
        toml::ser::to_string_pretty(&SETTINGS_MUT).unwrap()
    };
    let mut file = File::create("settings.toml").unwrap();
    file.write_all(settings_string.as_bytes()).unwrap();
}

impl Craft {
    pub fn run() {
        let mut events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title("")
            .with_dimensions(1280, 720);
        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Latest)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl_debug_flag(true)
            .with_vsync(true)
            .with_srgb(true);
        let ref display = glium::Display::new(window, context, &events_loop).unwrap();

        load_settings();
        let mut app = Craft::new(display);
        let mut imgui = ImGui::init();
        let mut imgui_renderer = ImGuiRenderer::init(&mut imgui, display).unwrap();

        let mut run = true;
        let mut test_window_opened = true;

        while run {
            events_loop.poll_events(|event| {
                match event {
                    Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                        run = false;
                    }
                    Event::WindowEvent {
                        event: WindowEvent::KeyboardInput {
                            input: KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(DEBUG_KEYCODE),
                                ..
                            }, ..
                        }, ..
                    } => {
                        if app.show_debug {
                            app.grab_cursor(display);
                            app.show_debug = false;
                        } else {
                            // If the cursor is currently captured we want to release it
                            app.release_cursor(display);
                            app.show_debug = true;
                        }
                    }
                    Event::WindowEvent {
                        event: WindowEvent::MouseInput {
                            state: ElementState::Pressed,
                            button: MouseButton::Left,
                            ..
                        }, ..
                    } if !app.show_debug && !app.mouse_grabbed  => {
                        app.grab_cursor(display);
                    }
                    Event::WindowEvent {
                        event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        app.width = width;
                        app.height = height;
                    }
                    event => {
                        if app.show_debug {
                            if let Event::WindowEvent { event, .. } = event {
                                app.imgui_on(&mut imgui, event);
                            }
                        } else if app.mouse_grabbed {
                            app.on(display, event);
                        }
                    }
                }
            });

            imgui.set_mouse_down(&app.mouse_buttons);
            let size_points = display.gl_window().get_inner_size_points().unwrap();
            let size_pixels = display.gl_window().get_inner_size_pixels().unwrap();
            let mut ui_frame = imgui.frame(size_points, size_pixels, 1.0 / 60.0);

            ui_frame.show_test_window(&mut test_window_opened);

            debug!("main update");
            app.update(display);

            let mut frame = display.draw();
            debug!("main render");
            app.render(display, &mut frame);
            imgui_renderer.render(&mut frame, ui_frame).unwrap();

            frame.finish().unwrap();
            debug!("main end frame");
        }

        store_settings();
    }

    fn new(display: &Display) -> Self {
        let (width, height) = display.gl_window().get_inner_size_pixels().unwrap();
        let aspect_ratio = (width as f32) / (height as f32);
        Craft {
            aspect_ratio: aspect_ratio,
            width, height,
            mouse_grabbed: false,
            show_debug: false,
            mouse_buttons: [false; 5],
            keys: [false; VirtualKeyCode::Yen as usize],

            line_renderer: LineRenderer::new(display),
            chunk_manager: ChunkManager::new(display, "save.sqlite".into()),
            tick: 0,
            player: Player::new(),
        }
    }

    fn update(&mut self, display: &Display) {
        unsafe {
            ui.window(im_str!("Settings")).build(|| {
                ui.input_int(im_str!("chunk_render_distance"), &mut SETTINGS_MUT.chunk_render_distance).build();
                ui.input_float(im_str!("far"), &mut SETTINGS_MUT.far).build();
                ui.input_float(im_str!("near"), &mut SETTINGS_MUT.near).build();
                ui.input_float(im_str!("mouse_sensitivity"), &mut SETTINGS_MUT.mouse_sensitivity).build();
                ui.input_float(im_str!("move_speed"), &mut SETTINGS_MUT.move_speed).step(0.01).build();
                ui.input_float(im_str!("reach_distance"), &mut SETTINGS_MUT.reach_distance).step(1.0).build();
                ui.input_float(im_str!("raycast_step_size"), &mut SETTINGS_MUT.raycast_step_size).build();
                ui.input_float(im_str!("raycast_max_distance"), &mut SETTINGS_MUT.raycast_max_distance).step(1.0).build();
            });
        }

        if self.keys[VirtualKeyCode::A as usize] {
            self.player.move_dir(Direction::Left);
        }
        if self.keys[VirtualKeyCode::D as usize] {
            self.player.move_dir(Direction::Right);
        }
        if self.keys[VirtualKeyCode::W as usize] {
            self.player.move_dir(Direction::Forward);
        }
        if self.keys[VirtualKeyCode::S as usize] {
            self.player.move_dir(Direction::Backward);
        }
        if self.keys[VirtualKeyCode::Space as usize] {
            self.player.move_dir(Direction::Up);
        }
        if self.keys[VirtualKeyCode::LControl as usize] {
            self.player.move_dir(Direction::Down);
        }

        ui.text(im_str!("{:?}", self.player.camera));
        self.chunk_manager.tick(display, self.player.camera);
        self.tick += 1;
    }

    fn render(&mut self, display: &Display, frame: &mut Frame) {
        let clip_from_view = perspective(Deg(45.0f32), self.aspect_ratio, 1.0, 1000.0);
        let view_from_world: Matrix4<f32> = Transform::look_at(
            self.player.camera.pos,
            self.player.camera.pos + self.player.camera.view().normalize(),
            Vector3::unit_y(),
        );
        let clip_from_world = clip_from_view * view_from_world ;

        frame.clear_color_and_depth((0.0, 1.0, 1.0, 1.0), 1.0);

        self.chunk_manager.render(frame, &clip_from_world);

        let casted = self.player.camera.raycast(
            &mut self.chunk_manager, SETTINGS.raycast_max_distance,
            SETTINGS.raycast_step_size, false
        );
        if let Some((pos, block)) = casted {
            let pos = Point3 { x: pos.x as f32, y: pos.y as f32, z: pos.z as f32 };
            self.draw_wireframe_cube(pos, display, frame, &clip_from_world);
        }

        self.draw_crosshair();
    }

    fn on(&mut self, display: &Display, event: Event) {
        match event {
            Event::WindowEvent { event, .. } => match event
            {
                WindowEvent::Resized(width, height) => {
                    self.aspect_ratio = (width as f32) / (height as f32);
                }
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        modifiers,
                        .. },
                    .. } => {
                    use glium::glutin::VirtualKeyCode::*;

                    let pressed = state == ElementState::Pressed;

                    self.keys[keycode as usize] = pressed;

                    trace!("Got input {:?}", keycode);
                    match keycode {
                        Escape if pressed => {
                            if self.mouse_grabbed {
                                self.mouse_grabbed = false;
                                display.gl_window().set_cursor_state(CursorState::Normal);
                            }
                        }
                        _ => {}
                    }
                }
                WindowEvent::Resized(width, height) => {
                    self.width = width;
                    self.height = height;
                }
                WindowEvent::MouseInput { button: MouseButton::Left, state: ElementState::Pressed, .. } => {
                    let casted = self.player.camera.raycast(
                        &mut self.chunk_manager, SETTINGS.raycast_max_distance,
                        SETTINGS.raycast_step_size, false
                    );
                    if let Some((pos, block)) = casted {
                        self.chunk_manager.set_block(pos, Block::new(BlockType::Air));
                    }
                }
                WindowEvent::MouseInput { button: MouseButton::Right, state: ElementState::Pressed, .. } => {
                    let casted = self.player.camera.raycast(
                        &mut self.chunk_manager, SETTINGS.raycast_max_distance,
                        SETTINGS.raycast_step_size, true
                    );
                    if let Some((pos, block)) = casted {
                        self.chunk_manager.set_block(pos, Block::new(BlockType::Stone));
                    }
                }
                _ => {}
            },
            Event::DeviceEvent { event: DeviceEvent::Motion { axis, value }, .. } => {
                use std::mem::transmute;
                let axis: u32 = unsafe { transmute(axis) };
                if axis == 0 {
                    self.player.camera.rotate_by(value as f32, 0.0);
                } else if axis == 1 {
                    self.player.camera.rotate_by(0.0, -value as f32);
                }
            }
            _ => {}
        }
    }
}

const DEBUG_KEYCODE: VirtualKeyCode = VirtualKeyCode::F2;

impl Craft {
    fn grab_cursor(&mut self, display: &Display) {
        display.gl_window().set_cursor_state(CursorState::Grab);
        self.mouse_grabbed = true;
        display.gl_window().set_cursor_position((self.width as i32) / 2, (self.height as i32) / 2);
    }

    fn release_cursor(&mut self, display: &Display) {
        display.gl_window().set_cursor_state(CursorState::Normal);
        self.mouse_grabbed = false;
    }

    fn imgui_on(&mut self, imgui: &mut ImGui, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input: KeyboardInput {
                state, virtual_keycode: Some(keycode), ..
            }, .. } => {
                use self::VirtualKeyCode::*;

                let pressed = state == ElementState::Pressed;

                match keycode {
                    Tab | Left | Right | Up | Down | PageUp | PageDown | Home | End | Delete |
                    Back | Return | Escape | A | C | V | X | Y | Z => {
                        imgui.set_key(keycode as u8, pressed);
                    }
                    LControl => imgui.set_key_ctrl(pressed),
                    LShift => imgui.set_key_shift(pressed),
                    LAlt => imgui.set_key_alt(pressed),
                    _ => {}
                }
            }
            WindowEvent::MouseMoved { position: (x, y), .. } => {
                imgui.set_mouse_pos(x as f32, y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                match button {
                    MouseButton::Left => self.mouse_buttons[0] = pressed,
                    MouseButton::Middle => self.mouse_buttons[1] = pressed,
                    MouseButton::Right => self.mouse_buttons[2] = pressed,
                    _ => {}
                }
            }
            WindowEvent::ReceivedCharacter(character) => {
                imgui.add_input_character(character);
            }
            //WindowEvent::MouseWheel {} => {}
            _ => {}
        }
    }

    fn draw_rect(id: &ImStr, centre_x: u32, centre_y: u32, width: u32, height: u32) {
        use imgui::*;

        let centre_x = centre_x as f32;
        let centre_y = centre_y as f32;
        let width = width as f32;
        let height = height as f32;

        let plain_window_flags: ImGuiWindowFlags = ImGuiWindowFlags_NoCollapse |
            ImGuiWindowFlags_NoInputs | ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoResize |
            ImGuiWindowFlags_NoSavedSettings | ImGuiWindowFlags_NoScrollWithMouse |
            ImGuiWindowFlags_NoScrollbar | ImGuiWindowFlags_NoTitleBar;

        static STYLE_VARS: [StyleVar; 3] = [
            StyleVar::WindowRounding(0.0),
            StyleVar::WindowMinSize(ImVec2 { x: 0.0, y: 0.0 }),
            StyleVar::WindowPadding(ImVec2 { x: 0.0, y: 0.0 })
        ];

        ui.with_color_var(ImGuiCol::WindowBg, ImVec4::new(0.4, 0.4, 0.4, 1.0), || {
            ui.with_style_vars(&STYLE_VARS, || {
                ui.window(id)
                    .flags(plain_window_flags)
                    .size((width, height), ImGuiSetCond_Always)
                    .position((centre_x - width / 2.0, centre_y - height / 2.0), ImGuiSetCond_Always)
                    .bg_alpha(0.8)
                    .build(|| {});
            })
        })
    }

    fn draw_crosshair(&self) {
        let centre_x = self.width / 2;
        let centre_y = self.height / 2;

        Self::draw_rect(im_str!("##1"), centre_x, centre_y, 32, 3);
        Self::draw_rect(im_str!("##2"), centre_x, centre_y, 3, 32);
    }

    fn draw_wireframe_cube(&mut self, pos: Point3<f32>, display: &Display, frame: &mut Frame, clip_from_world: &Matrix4<f32>) {
        const L: f32 = -0.005;
        const R: f32 = 1.005;
        static CUBE_WIREFRAME: [(Vector3, Vector3); 12] = [
            (Vector3 { x: L, y: L, z: L }, Vector3 { x: L, y: L, z: R }),
            (Vector3 { x: L, y: L, z: L }, Vector3 { x: L, y: R, z: L }),
            (Vector3 { x: L, y: L, z: L }, Vector3 { x: R, y: L, z: L }),
            (Vector3 { x: L, y: L, z: R }, Vector3 { x: R, y: L, z: R }),
            (Vector3 { x: L, y: L, z: R }, Vector3 { x: L, y: R, z: R }),
            (Vector3 { x: L, y: R, z: L }, Vector3 { x: R, y: R, z: L }),
            (Vector3 { x: L, y: R, z: L }, Vector3 { x: L, y: R, z: R }),
            (Vector3 { x: L, y: R, z: R }, Vector3 { x: R, y: R, z: R }),
            (Vector3 { x: R, y: L, z: L }, Vector3 { x: R, y: R, z: L }),
            (Vector3 { x: R, y: L, z: L }, Vector3 { x: R, y: L, z: R }),
            (Vector3 { x: R, y: L, z: R }, Vector3 { x: R, y: R, z: R }),
            (Vector3 { x: R, y: R, z: L }, Vector3 { x: R, y: R, z: R }),
        ];

        for &(a, b) in CUBE_WIREFRAME.iter() {
            let start = pos + a;
            let end = pos + b;
            self.line_renderer.line3d(start, end, Color::new(0, 0, 0, 0));
        }

        self.line_renderer.render(display, frame, &clip_from_world, 2.0);
    }
}
