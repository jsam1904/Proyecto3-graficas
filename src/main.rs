//! Renderizador por software: sistema solar de Star Wars explorable.

mod app;
mod camera;
mod config;
mod culling;
mod framebuffer;
mod hud;
mod math;
mod mesh;
mod noise;
mod obj;
mod pipeline;
mod planet;
mod placeholders;
mod platform;
mod raster;
mod scene;
mod shaders;
mod ship;
mod stars;
mod surface;
mod texture;

use app::App;
use camera::Camera;
use framebuffer::Framebuffer;
use math::*;
use platform::{Input, Key, Platform};

/// Opciones de línea de comandos.
struct Args {
    screenshot: Option<String>,
    frames: u32,
    seed: u64,
    cam: Option<Camera>,
    wire: bool,
    hold: Vec<Key>,
    /// Segundos de simulación a avanzar con dt fijo antes de la captura (equivale a --frames N con dt=1/60).
    flat: bool,
    fp: bool,
    ship_pos: Option<Vec3>,
    nolod: bool,
    nocull: bool,
    nohud: bool,
    bench: u32,
    record: Option<String>,
    every: u32,
    planet: usize,
    time: f32,
}

fn parse_args() -> Args {
    let mut a = Args { screenshot: None, frames: 1, seed: config::SEED_DEFAULT, cam: None, wire: false, hold: vec![], flat: false, fp: false, ship_pos: None, nolod: false, nocull: false, nohud: false, bench: 0, record: None, every: 3, planet: 0, time: 0.0 };
    let v: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < v.len() {
        match v[i].as_str() {
            "--screenshot" => { a.screenshot = v.get(i + 1).cloned(); i += 1; }
            "--frames" => { a.frames = v[i + 1].parse().expect("--frames N"); i += 1; }
            "--seed" => { a.seed = v[i + 1].parse().expect("--seed N"); i += 1; }
            "--cam" => {
                // "x,y,z,yaw,pitch" con ángulos en grados.
                let n: Vec<f32> = v[i + 1].split(',').map(|s| s.trim().parse().expect("--cam x,y,z,yaw,pitch")).collect();
                assert!(n.len() == 5, "--cam necesita 5 valores: x,y,z,yaw,pitch");
                a.cam = Some(Camera::from_yaw_pitch(vec3(n[0], n[1], n[2]), n[3].to_radians(), n[4].to_radians()));
                i += 1;
            }
            "--hold" => { a.hold = v[i + 1].split(',').map(|s| Key::from_name(s).expect("tecla desconocida en --hold")).collect(); i += 1; }
            "--wire" => a.wire = true,
            "--flat" => a.flat = true,
            "--fp" => a.fp = true,
            "--nolod" => a.nolod = true,
            "--nocull" => a.nocull = true,
            "--nohud" => a.nohud = true,
            "--record" => { a.record = v.get(i + 1).cloned(); i += 1; }
            "--every" => { a.every = v[i + 1].parse().expect("--every K"); i += 1; }
            "--bench" => { a.bench = v[i + 1].parse().expect("--bench N"); i += 1; }
            "--planet" => { a.planet = v[i + 1].parse().expect("--planet i"); i += 1; }
            "--time" => { a.time = v[i + 1].parse().expect("--time T"); i += 1; }
            "--ship" => {
                let n: Vec<f32> = v[i + 1].split(',').map(|s| s.trim().parse().expect("--ship x,y,z")).collect();
                a.ship_pos = Some(vec3(n[0], n[1], n[2]));
                i += 1;
            }
            _ => eprintln!("argumento ignorado: {}", v[i]),
        }
        i += 1;
    }
    a
}

fn main() {
    let args = parse_args();
    let mut app = App::new(args.seed);
    app.wire = args.wire;
    app.cam_override = args.cam;
    app.flat_shading = args.flat;
    app.cam.first_person = args.fp;
    app.lod = !args.nolod;
    app.cull = !args.nocull;
    app.hud = !args.nohud;
    app.t = args.time;
    if let Some(p) = args.ship_pos { app.place_ship(p, args.planet); }
    let mut fb = Framebuffer::new(config::RENDER_W, config::RENDER_H);
    let mut fps = hud::FpsCounter::new();

    // Medición de rendimiento sin ventana: N cuadros con dt fijo y teclas simuladas; imprime ms/cuadro promedio.
    if args.bench > 0 {
        let mut inp = Input::vacio();
        for k in &args.hold { inp.hold(*k); }
        let (mut total, mut peor) = (0.0f32, 0.0f32);
        let mut st = pipeline::Stats::default();
        for _ in 0..args.bench {
            app.update(&inp, config::DT_SCREENSHOT);
            let t0 = std::time::Instant::now();
            st = app.render(&mut fb, &fps);
            let ms = t0.elapsed().as_secs_f32() * 1000.0;
            total += ms;
            peor = peor.max(ms);
        }
        let prom = total / args.bench as f32;
        println!("bench: {} cuadros, {:.2} ms/cuadro promedio ({:.0} FPS), peor {:.1} ms | tris enviados {} rasterizados {} | objetos {}/{} (frustum {}, lod-omit {})",
            args.bench, prom, 1000.0 / prom, peor, st.tris_sent, st.tris_raster, st.objs_drawn, st.objs_total, st.objs_culled, st.objs_lod_omit);
        return;
    }

    // Grabación sin ventana: simula --frames cuadros y guarda uno de cada --every como PPM numerado (para armar un GIF/video).
    if let Some(prefijo) = &args.record {
        let mut inp = Input::vacio();
        for k in &args.hold { inp.hold(*k); }
        fps.fps = 60.0;
        fps.ms = 16.7;
        let mut n = 0;
        for f in 0..args.frames {
            app.update(&inp, config::DT_SCREENSHOT);
            if f % args.every == 0 {
                app.render(&mut fb, &fps);
                fb.save_ppm(&format!("{prefijo}_{n:04}.ppm")).expect("no se pudo guardar el cuadro");
                n += 1;
            }
        }
        println!("{n} cuadros guardados con prefijo {prefijo}");
        return;
    }

    // Modo sin ventana: dt fijo, teclas simuladas con --hold, guarda el último cuadro.
    if let Some(path) = &args.screenshot {
        let mut inp = Input::vacio();
        for k in &args.hold { inp.hold(*k); }
        for _ in 0..args.frames { app.update(&inp, config::DT_SCREENSHOT); }
        fps.fps = 60.0;
        fps.ms = 16.7;
        let t0 = std::time::Instant::now();
        let st = app.render(&mut fb, &fps);
        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        println!("render: {:.1} ms ({} tris enviados, {} rasterizados)", ms, st.tris_sent, st.tris_raster);
        fb.save_ppm(path).expect("no se pudo guardar la captura");
        let (cuerpo, alt) = app.nearest_altitude();
        println!("captura en {path}: t={:.1}s pos=({:.1},{:.1},{:.1}) vel={:.1} u/s alt={:.2} sobre {cuerpo} g={:.1} modo={}", app.t, app.ship.pos.x, app.ship.pos.y, app.ship.pos.z, app.ship.speed(), alt, app.ship.grav.length(), match app.local { Some(i) => config::PLANETAS[i].nombre, None => "ESPACIAL" });
        return;
    }

    let mut plat = Platform::new(config::WINDOW_W, config::WINDOW_H, fb.w, fb.h, config::FPS_LIMIT, "Sistema Solar - Software Renderer");
    while !plat.should_close() {
        let dt = plat.frame_time().min(config::DT_MAX);
        let input = plat.poll();
        if input.pressed(Key::Esc) { break; }
        fps.tick(dt);
        app.update(&input, dt);
        app.render(&mut fb, &fps);
        plat.present(&fb);
    }
}
