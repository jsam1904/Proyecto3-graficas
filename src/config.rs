//! Parámetros ajustables del proyecto, todos en un solo lugar.

// --- Ventana y resolución interna ---
/// Tamaño de la ventana (el framebuffer interno se escala a este tamaño).
pub const WINDOW_W: i32 = 1280;
pub const WINDOW_H: i32 = 720;
/// Resolución interna del renderizador (RENDER_SCALE: menos píxeles = más FPS).
pub const RENDER_W: usize = 960;
pub const RENDER_H: usize = 540;
/// Hilos de rasterización (0 = todos los núcleos disponibles, 1 = un solo hilo) y filas por banda.
pub const RENDER_THREADS: usize = 0;
pub const BAND_ROWS: usize = 12;
/// Límite de FPS de la ventana (0 = sin límite).
pub const FPS_LIMIT: u32 = 120;

// --- Tiempo ---
/// Máximo dt aceptado por cuadro (evita saltos tras una pausa larga).
pub const DT_MAX: f32 = 0.05;
/// dt fijo usado en el modo --screenshot (reproducible).
pub const DT_SCREENSHOT: f32 = 1.0 / 60.0;

// --- Cámara / proyección ---
pub const FOV_Y_DEG: f32 = 60.0;
pub const NEAR: f32 = 0.1;
/// FOV en modo local y rapidez de la transición (1/s).
pub const FOV_LOCAL_DEG: f32 = 80.0;
pub const FOV_SMOOTH: f32 = 3.0;
pub const FAR: f32 = 5000.0;

// --- Mallas ---
pub const SPHERE_OBJ: &str = "assets/sphere.obj";

// --- Nave ---
pub const SHIP_OBJ: &str = "assets/Nave_Javier_Alvarado.obj";
/// Largo deseado de la nave en unidades del mundo.
pub const SHIP_LENGTH: f32 = 2.0;
/// Multiplicador extra de escala.
pub const SHIP_SCALE: f32 = 1.0;
/// Rotación (grados X,Y,Z) que lleva el "adelante" del modelo a -Z. A verificar con capturas.
pub const SHIP_FORWARD_FIX: [f32; 3] = [0.0, 180.0, 0.0];

// --- Nave: física y controles (unidades/segundo) ---
/// Posición inicial y hacia qué cuerpo (índice en PLANETAS) mira al empezar.
pub const SHIP_START_POS: [f32; 3] = [380.0, 30.0, 280.0];
pub const SHIP_START_LOOK_PLANET: usize = 0;
pub const SHIP_CRUISE_SPEED: f32 = 60.0;
pub const SHIP_TURBO_SPEED: f32 = 400.0;
/// Aceleración con W (crucero / turbo).
pub const SHIP_ACCEL: f32 = 90.0;
pub const SHIP_ACCEL_TURBO: f32 = 500.0;
/// Amortiguación (1/s): con empuje es pequeña; sin empuje o con S la nave frena.
pub const SHIP_DRAG_THRUST: f32 = 0.05;
pub const SHIP_DRAG_COAST: f32 = 0.6;
pub const SHIP_DRAG_BRAKE: f32 = 2.5;
/// Velocidades angulares máximas (rad/s) de guiñada, cabeceo y alabeo.
pub const SHIP_YAW_RATE: f32 = 1.2;
pub const SHIP_PITCH_RATE: f32 = 1.0;
pub const SHIP_ROLL_RATE: f32 = 1.8;
/// Rapidez con que la velocidad angular alcanza la deseada (1/s). Más alto = más ágil.
pub const SHIP_ANG_SMOOTH: f32 = 5.0;
/// true = flecha arriba baja la nariz.
pub const INVERT_PITCH: bool = false;

// --- Cámara de seguimiento ---
pub const CAM_DIST: f32 = 6.0;
pub const CAM_HEIGHT: f32 = 1.8;
/// Cuánto se aleja la cámara a velocidad turbo (fracción de CAM_DIST).
pub const CAM_SPEED_PULLBACK: f32 = 0.8;
/// Rigidez del resorte de seguimiento (1/s). Más alto = más pegada a la nave.
pub const CAM_FOLLOW_K: f32 = 5.0;
pub const CAM_UP_K: f32 = 3.0;
pub const CAM_LOOK_AHEAD: f32 = 25.0;
/// Mirar alrededor: velocidad con flechas izq/der (rad/s), sensibilidad del mouse (rad/píxel, botón derecho) y retorno al centro (1/s).
pub const LOOK_KEY_RATE: f32 = 2.0;
pub const LOOK_MOUSE_SENS: f32 = 0.004;
pub const LOOK_RECENTER: f32 = 1.5;
/// En modo local la cámara mira un poco hacia el planeta (unidades hacia "abajo" local del punto de mira).
pub const CAM_LOCAL_DOWN: f32 = 30.0;

// --- Tiempo de simulación ---
/// Escalas de tiempo disponibles con [ y ] (afecta órbitas y rotaciones, no la nave).
pub const TIME_SCALES: [f32; 7] = [0.25, 0.5, 1.0, 2.0, 5.0, 20.0, 100.0];
pub const TIME_SCALE_DEFAULT: usize = 2;

// --- Estrellas ---
pub const STAR_COUNT: usize = 1800;
pub const STAR_SEED: u32 = 12345;

// --- Sistema solar ---
pub const SUN_RADIUS: f32 = 120.0;

/// Tipo de relieve (ver planet::altura_norm).
#[derive(Clone, Copy)]
pub enum Terreno { Dunas, Montanas, Colinas }

/// Shader de fragmentos de cada planeta (ver shaders/mod.rs).
#[derive(Clone, Copy)]
pub enum ShaderKind { Tatooine, Hoth, Endor, Mustafar, Coruscant, DeathStar }

pub struct PlanetCfg {
    pub nombre: &'static str,
    pub radio: f32,
    /// Radio de la órbita (circular, en el plano XZ) y velocidad angular (rad/s) y fase inicial (rad).
    pub orbita: f32,
    pub vel_orbita: f32,
    pub fase: f32,
    /// Velocidad de rotación sobre su eje (rad/s) e inclinación axial (grados).
    pub vel_rotacion: f32,
    pub inclinacion_deg: f32,
    /// Masa (para la gravedad g = G·masa/(r²+ε)).
    pub masa: f32,
    /// Relieve: tipo, altura máxima como fracción del radio, frecuencia del ruido, desfase (semilla del planeta)
    /// y nivel del mar (0..1; el relieve por debajo queda plano, p. ej. océanos).
    pub terreno: Terreno,
    pub amplitud: f32,
    pub frecuencia: f32,
    pub semilla: [f32; 3],
    pub nivel_mar: f32,
    pub shader: ShaderKind,
    /// Color del halo atmosférico fuera del disco (negro = sin atmósfera).
    pub atmosfera: [f32; 3],
}

pub const PLANETAS: [PlanetCfg; 6] = [
    PlanetCfg { nombre: "TATOOINE", radio: 40.0, orbita: 450.0, vel_orbita: 0.020, fase: 0.0, vel_rotacion: 0.25, inclinacion_deg: 12.0,
                masa: 64.0, terreno: Terreno::Dunas, amplitud: 0.05, frecuencia: 6.0, semilla: [3.1, 7.7, 1.3], nivel_mar: 0.0, shader: ShaderKind::Tatooine, atmosfera: [0.45, 0.65, 1.0] },
    PlanetCfg { nombre: "HOTH", radio: 36.0, orbita: 650.0, vel_orbita: 0.016, fase: 2.0, vel_rotacion: 0.20, inclinacion_deg: 23.0,
                masa: 55.0, terreno: Terreno::Montanas, amplitud: 0.08, frecuencia: 5.0, semilla: [9.4, 2.2, 5.8], nivel_mar: 0.0, shader: ShaderKind::Hoth, atmosfera: [0.55, 0.75, 1.0] },
    PlanetCfg { nombre: "ENDOR", radio: 48.0, orbita: 850.0, vel_orbita: 0.013, fase: 3.6, vel_rotacion: 0.18, inclinacion_deg: 8.0,
                masa: 90.0, terreno: Terreno::Colinas, amplitud: 0.05, frecuencia: 3.5, semilla: [1.9, 4.6, 8.3], nivel_mar: 0.42, shader: ShaderKind::Endor, atmosfera: [0.35, 0.60, 1.0] },
    PlanetCfg { nombre: "MUSTAFAR", radio: 32.0, orbita: 1050.0, vel_orbita: 0.011, fase: 5.0, vel_rotacion: 0.22, inclinacion_deg: 0.0,
                masa: 50.0, terreno: Terreno::Montanas, amplitud: 0.07, frecuencia: 6.0, semilla: [6.6, 0.8, 3.9], nivel_mar: 0.0, shader: ShaderKind::Mustafar, atmosfera: [1.0, 0.30, 0.10] },
    PlanetCfg { nombre: "CORUSCANT", radio: 56.0, orbita: 1250.0, vel_orbita: 0.009, fase: 1.0, vel_rotacion: 0.12, inclinacion_deg: 18.0,
                masa: 120.0, terreno: Terreno::Colinas, amplitud: 0.012, frecuencia: 3.0, semilla: [2.5, 9.1, 6.2], nivel_mar: 0.0, shader: ShaderKind::Coruscant, atmosfera: [0.55, 0.70, 1.0] },
    // Estrella de la Muerte: esfera lisa (amplitud 0), sin atmósfera, rota lento.
    PlanetCfg { nombre: "DEATH STAR", radio: 34.0, orbita: 1420.0, vel_orbita: 0.0075, fase: 4.3, vel_rotacion: 0.03, inclinacion_deg: 0.0,
                masa: 100.0, terreno: Terreno::Colinas, amplitud: 0.0, frecuencia: 1.0, semilla: [0.0, 0.0, 0.0], nivel_mar: 0.0, shader: ShaderKind::DeathStar, atmosfera: [0.0, 0.0, 0.0] },
];

// --- Parámetros de shaders (ajustables en vivo para la defensa) ---
/// Velocidad de la animación de la granulación solar.
pub const SUN_ANIM_SPEED: f32 = 0.12;
/// Intensidad del mapa de normales de Tatooine y repeticiones de la textura por cara.
pub const NORMAL_MAP_STRENGTH: f32 = 0.2;
pub const NORMAL_MAP_TILING: f32 = 12.0;
pub const NORMAL_MAP_SIZE: usize = 256;
/// Brillo de las grietas de lava de Mustafar y su velocidad de pulso.
pub const LAVA_INTENSITY: f32 = 1.2;
pub const LAVA_PULSE_SPEED: f32 = 1.6;
/// Brillo de los nodos urbanos de Coruscant y densidad de nodos (frecuencia del ruido de Worley).
pub const CITY_LIGHT_INTENSITY: f32 = 1.2;
pub const CITY_GRID: f32 = 3.6;
/// Intensidad de los destellos especulares y del borde Fresnel de Hoth.
pub const ICE_GLINT: f32 = 1.0;
pub const ICE_RIM: f32 = 1.0;
/// Cobertura de nubes de Endor (0..1) y su velocidad.
pub const CLOUD_COVER: f32 = 0.5;
pub const CLOUD_SPEED: f32 = 0.01;

// --- Gravedad y colisión ---
/// Constante gravitacional "artística" y suavizado ε (evita g infinita cerca del centro).
pub const G: f32 = 1000.0;
pub const GRAVITY_EPS: f32 = 100.0;
/// Masa del Sol (su atracción es mucho más débil que la de un planeta cercano).
pub const SUN_MASS: f32 = 400.0;
/// Radio de colisión de la nave y rebote (0 = se pega, 1 = rebote perfecto) al tocar un cuerpo.
pub const SHIP_COLLISION_RADIUS: f32 = 1.0;
pub const COLLISION_BOUNCE: f32 = 0.15;
/// Separación mínima de la cámara a la superficie.
pub const CAM_SURFACE_MARGIN: f32 = 0.8;

// --- LOD ---
/// Umbrales de distancia (en radios del cuerpo): más lejos que el 1º => nivel 0 (192 tris), ..., más cerca que el último => nivel 3.
pub const LOD_DIST: [f32; 3] = [14.0, 6.0, 2.5];
pub const LOD_LEVELS: usize = 4;

// --- Objetos de superficie ---
/// Semilla por defecto de la distribución procedural (se cambia con --seed N). Misma semilla => mismas posiciones.
pub const SEED_DEFAULT: u64 = 1;
/// Carpeta de modelos de superficie (los placeholders se generan aquí si faltan).
pub const SURFACE_DIR: &str = "assets/surface";
/// Los objetos de superficie solo se dibujan si la cámara está a menos de este múltiplo del radio del planeta.
pub const SURFACE_DRAW_DIST: f32 = 6.0;

/// Un tipo de modelo colocado sobre un planeta.
pub struct SurfaceCfg {
    /// Índice del planeta en PLANETAS y archivo OBJ.
    pub planeta: usize,
    pub archivo: &'static str,
    /// Máximo de instancias, rango de alturas (unidades del mundo) y separación mínima como fracción de la altura.
    pub cantidad: usize,
    pub altura: [f32; 2],
    pub separacion: f32,
    /// Agrupamiento: frecuencia del fBm y umbral (más alto = grupos más pequeños y escasos).
    pub frecuencia: f32,
    pub umbral: f32,
    /// Cuánto se hunde la base en el terreno (unidades) para que no flote en las pendientes.
    pub hundir: f32,
    /// Color de la parte baja y alta, y fracción de altura donde cambia.
    pub color_bajo: [f32; 3],
    pub color_alto: [f32; 3],
    pub corte: f32,
}

pub const SUPERFICIE: [SurfaceCfg; 4] = [
    SurfaceCfg { planeta: 0, archivo: "vaporizador.obj", cantidad: 90, altura: [2.4, 3.6], separacion: 2.0, frecuencia: 2.5, umbral: 0.30, hundir: 0.15,
                 color_bajo: [0.45, 0.42, 0.38], color_alto: [0.78, 0.76, 0.70], corte: 0.14 },
    SurfaceCfg { planeta: 1, archivo: "pico_hielo.obj", cantidad: 120, altura: [1.8, 4.5], separacion: 0.6, frecuencia: 3.0, umbral: 0.42, hundir: 0.3,
                 color_bajo: [0.50, 0.65, 0.85], color_alto: [0.90, 0.96, 1.0], corte: 0.5 },
    SurfaceCfg { planeta: 2, archivo: "arbol.obj", cantidad: 200, altura: [3.0, 6.0], separacion: 0.7, frecuencia: 3.0, umbral: 0.40, hundir: 0.3,
                 color_bajo: [0.30, 0.20, 0.10], color_alto: [0.08, 0.30, 0.10], corte: 0.34 },
    SurfaceCfg { planeta: 4, archivo: "rascacielos.obj", cantidad: 260, altura: [1.5, 4.5], separacion: 0.7, frecuencia: 2.0, umbral: 0.30, hundir: 0.3,
                 color_bajo: [0.10, 0.075, 0.06], color_alto: [0.46, 0.30, 0.18], corte: 0.68 },
];

// --- Modo de control local (se activa solo al entrar en la zona de influencia de un planeta) ---
/// Entra a modo local a menos de ENTER radios del centro del planeta y sale a más de EXIT (histéresis: evita parpadeo).
pub const LOCAL_ENTER: f32 = 2.2;
pub const LOCAL_EXIT: f32 = 2.6;
/// Tope de velocidad relativa al planeta (normal / turbo).
pub const LOCAL_SPEED: f32 = 40.0;
pub const LOCAL_TURBO_SPEED: f32 = 120.0;
/// La gravedad se atenúa en modo local (la nave "flota" y se puede rodear el planeta sin caer).
pub const LOCAL_GRAVITY_SCALE: f32 = 0.2;
/// Rapidez con que el "arriba" de la nave se alinea con la vertical local (1/s).
pub const LOCAL_ALIGN_K: f32 = 2.5;
/// Velocidad vertical con Espacio/Ctrl (u/s) y rapidez con que la velocidad vertical converge (1/s).
pub const LOCAL_CLIMB_SPEED: f32 = 28.0;
pub const LOCAL_VERT_K: f32 = 3.0;
/// Ganancia de la retención de altitud (1/s): velocidad radial = error de distancia · K.
pub const LOCAL_ALT_K: f32 = 0.8;

// --- LOD y simplificación de objetos de superficie ---
/// Distancias (en alturas del objeto) para pasar de malla completa a simple y de simple a omitido.
pub const LOD_OBJ_DIST: [f32; 2] = [14.0, 45.0];
/// Tamaño de celda (el modelo mide 1 de alto) de la simplificación por clustering de vértices.
pub const SIMPLIFY_CELL: f32 = 0.34;
