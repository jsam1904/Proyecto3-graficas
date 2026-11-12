//! Única capa que toca raylib: ventana, entrada, tiempo y subir el framebuffer como textura.
//! Si raylib no compila, solo hay que reescribir este archivo (p. ej. con minifb).

use crate::framebuffer::Framebuffer;
use raylib::prelude::*;

/// Teclas que usa el juego (independientes de raylib).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum Key {
    W, A, S, D, Q, E, Up, Down, Left, Right, Shift, Ctrl, Space,
    C, G, P, LBracket, RBracket, F1, F2, F3, F4, Esc,
}
const N_KEYS: usize = 23;

const MAPA: [(Key, KeyboardKey); N_KEYS] = [
    (Key::W, KeyboardKey::KEY_W), (Key::A, KeyboardKey::KEY_A), (Key::S, KeyboardKey::KEY_S),
    (Key::D, KeyboardKey::KEY_D), (Key::Q, KeyboardKey::KEY_Q), (Key::E, KeyboardKey::KEY_E),
    (Key::Up, KeyboardKey::KEY_UP), (Key::Down, KeyboardKey::KEY_DOWN),
    (Key::Left, KeyboardKey::KEY_LEFT), (Key::Right, KeyboardKey::KEY_RIGHT),
    (Key::Shift, KeyboardKey::KEY_LEFT_SHIFT), (Key::Ctrl, KeyboardKey::KEY_LEFT_CONTROL),
    (Key::Space, KeyboardKey::KEY_SPACE), (Key::C, KeyboardKey::KEY_C), (Key::G, KeyboardKey::KEY_G), (Key::P, KeyboardKey::KEY_P),
    (Key::LBracket, KeyboardKey::KEY_LEFT_BRACKET), (Key::RBracket, KeyboardKey::KEY_RIGHT_BRACKET),
    (Key::F1, KeyboardKey::KEY_F1), (Key::F2, KeyboardKey::KEY_F2), (Key::F3, KeyboardKey::KEY_F3),
    (Key::F4, KeyboardKey::KEY_F4), (Key::Esc, KeyboardKey::KEY_ESCAPE),
];

/// Estado de entrada de un cuadro.
#[derive(Clone)]
pub struct Input {
    down: [bool; N_KEYS],
    pressed: [bool; N_KEYS],
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    /// Botón derecho mantenido (se usa para mirar alrededor con el mouse).
    pub mouse_right: bool,
}

impl Key {
    /// Nombre de tecla -> Key (para `--hold` en el modo de prueba sin ventana).
    pub fn from_name(n: &str) -> Option<Key> {
        Some(match n.to_ascii_uppercase().as_str() {
            "W" => Key::W, "A" => Key::A, "S" => Key::S, "D" => Key::D, "Q" => Key::Q, "E" => Key::E,
            "UP" => Key::Up, "DOWN" => Key::Down, "LEFT" => Key::Left, "RIGHT" => Key::Right,
            "SHIFT" => Key::Shift, "CTRL" => Key::Ctrl, "SPACE" => Key::Space, _ => return None,
        })
    }
}

impl Input {
    /// Marca una tecla como mantenida (pruebas sin ventana).
    pub fn hold(&mut self, k: Key) { self.down[k as usize] = true; }
    pub fn vacio() -> Self {
        Self { down: [false; N_KEYS], pressed: [false; N_KEYS], mouse_dx: 0.0, mouse_dy: 0.0, mouse_right: false }
    }
    /// Tecla mantenida.
    pub fn down(&self, k: Key) -> bool { self.down[k as usize] }
    /// Tecla recién presionada en este cuadro.
    pub fn pressed(&self, k: Key) -> bool { self.pressed[k as usize] }
}

pub struct Platform {
    rl: RaylibHandle,
    thread: RaylibThread,
    texture: Texture2D,
    bytes: Vec<u8>,
    fb_w: i32,
    fb_h: i32,
}

impl Platform {
    pub fn new(win_w: i32, win_h: i32, fb_w: usize, fb_h: usize, fps: u32, title: &str) -> Self {
        let (mut rl, thread) = raylib::init().size(win_w, win_h).title(title).build();
        if fps > 0 {
            rl.set_target_fps(fps);
        }
        rl.set_exit_key(None); // Esc lo manejamos nosotros
        let img = Image::gen_image_color(fb_w as i32, fb_h as i32, Color::BLACK);
        let texture = rl.load_texture_from_image(&thread, &img).expect("no se pudo crear la textura");
        Self { rl, thread, texture, bytes: Vec::new(), fb_w: fb_w as i32, fb_h: fb_h as i32 }
    }

    pub fn should_close(&self) -> bool { self.rl.window_should_close() }

    /// Segundos que tardó el cuadro anterior (medido por raylib).
    pub fn frame_time(&self) -> f32 { self.rl.get_frame_time() }

    /// Lee teclado y mouse.
    pub fn poll(&mut self) -> Input {
        let mut inp = Input::vacio();
        for (k, rk) in MAPA {
            inp.down[k as usize] = self.rl.is_key_down(rk);
            inp.pressed[k as usize] = self.rl.is_key_pressed(rk);
        }
        let d = self.rl.get_mouse_delta();
        inp.mouse_dx = d.x;
        inp.mouse_dy = d.y;
        inp.mouse_right = self.rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);
        inp
    }

    /// Sube el framebuffer como textura 2D y la muestra escalada a la ventana.
    pub fn present(&mut self, fb: &Framebuffer) {
        fb.to_rgba(&mut self.bytes);
        self.texture.update_texture(&self.bytes).expect("update_texture");
        let (ww, wh) = (self.rl.get_screen_width() as f32, self.rl.get_screen_height() as f32);
        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &self.texture,
            Rectangle::new(0.0, 0.0, self.fb_w as f32, self.fb_h as f32),
            Rectangle::new(0.0, 0.0, ww, wh),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
    }
}
