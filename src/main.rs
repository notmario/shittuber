use std::time::Duration;
use macroquad::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::HashMap;
use std::fs;
use toml::Table;

const BALLS_SIZE: usize = 8;

static mut BALLS: [f32; BALLS_SIZE] = [0.;BALLS_SIZE];
static mut BALLSPOINT: usize = 0;

fn add_to_balls(data: f32) {
    unsafe {
        BALLS[BALLSPOINT] = data;
        if BALLSPOINT < BALLS_SIZE - 1 {
            BALLSPOINT += 1
        } else {
            BALLSPOINT = 0
        }
    }
}

fn read_balls() -> f32 {
    let mut sum = 0.;
    unsafe {
        for dat in BALLS {
            sum += dat
        }
    }
    sum / BALLS_SIZE as f32
}

enum State {
    Select(usize, f32),
    Main(String),
    // Settings(Option<String>),
}

struct Settings {
    thresh: f32,
    coyote_time: f32,
    move_mod: f32,
}

enum TalkMode {
    None,
    MoveUp(f32),
    Shake(f32),
}

struct Avatar {
    idle: Texture2D,
    idlesize: (f32, f32),
    speak: Texture2D,
    speaksize: (f32, f32),
    bgcol: Color,
    talkmode: TalkMode,
}

/// initial window stuff
fn window_conf() -> Conf {
    Conf {
        window_title: "shittuber".to_owned(),
        fullscreen: false,
        window_width: 1280,
        window_height: 720,
        window_resizable: false,

        ..Default::default()
    }
}

async fn my_load_texture(s: String, tex: &mut HashMap<String, Texture2D>, mode: FilterMode) -> Result<Texture2D,macroquad::Error> {
    if !tex.contains_key(&s) {
        let te = load_texture(&s).await?;
        te.set_filter(mode);
        tex.insert(s, te.clone());
        Ok(te.clone())
    } else {
        Ok(tex.get(&s).unwrap().clone())
    }
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    let host = cpal::default_host();
    
    let text_col = Color::from_hex(0xcad3f5);

    let mut selected_ind = 0;

    let devs: Vec<cpal::Device> = 
        host.devices().expect("I CANT EVEN")
        .filter(|a| a.supported_input_configs().is_ok_and(|k| k.count() > 0)).collect();

    loop {
        clear_background(Color::from_hex(0x1e2030));

        draw_text("CHOOSE YOUR AUDIO DEVICE (ARROWS + Z)", 4., 32. - 4., 32., text_col);
        draw_text("if yours doesnt show up SHOUT AT ME LOUDLY", 4., 64. - 4., 32., text_col);

        for (i, d) in devs.iter().enumerate() {
            let string = if i == selected_ind {
                format!("=> {}: {}", i, d.name().unwrap_or("MYSTERY SHIT".into()))
            } else {
                format!("   {}: {}", i, d.name().unwrap_or("MYSTERY SHIT".into()))
            };
            draw_text(&string, 4., (4 + i) as f32 * 32. - 4., 32., text_col);
        }
        
        if is_key_pressed(KeyCode::Z) {
            next_frame().await;
            break
        }

        if is_key_pressed(KeyCode::Down) {
            if selected_ind == devs.len() - 1 {
                selected_ind = 0
            } else {
                selected_ind += 1
            }
        }
        if is_key_pressed(KeyCode::Up) {
            if selected_ind == 0 {
                selected_ind = devs.len() - 1
            } else {
                selected_ind -= 1
            }
        }

        next_frame().await;
    };

    let device = devs[selected_ind].clone();
    let mut supported_configs_range = device.supported_input_configs()
        .expect("we have confirmed this to be true");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let stream = device.build_input_stream(
        &supported_config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            add_to_balls(data[0].abs());
            add_to_balls(data[1].abs());
            add_to_balls(data[2].abs());
            add_to_balls(data[3].abs());
        },
        move |_| {
            // uh?
        },
        Some(Duration::from_millis(1000)) // None=blocking, Some(Duration)=timeout
    ).expect("this should exist");
    stream.play().unwrap();
    
    let mut textures: HashMap<String, Texture2D> = HashMap::new();

    let avatar_dirs = fs::read_dir("avatars")?
    .map(|res| res.map(|e| e.path()))
    .filter_map(|res| res.ok());

    let mut avatars: HashMap<String, Avatar> = HashMap::new();
    for dir in avatar_dirs {
        if !dir.is_dir() { continue };

        let meta = std::fs::read_to_string(dir.join("meta.toml"));
        let Ok(meta) = meta else { continue };
        let meta = meta.parse::<Table>();
        let Ok(meta) = meta else { continue };

        let filtermode = if meta.contains_key("filtermode") {
            if meta["filtermode"].as_str().unwrap_or("asdf") == "nearest" {
                FilterMode::Nearest
            } else {
                FilterMode::Linear
            }
        } else {
            FilterMode::Linear
        };

        let paths = ["idle.png", "speak.png"];
        let mut imgs = vec![];
        for path in paths {
            let img = my_load_texture(dir.join(path).to_str().expect("should exist").into(), &mut textures, filtermode).await;
            let Ok(img) = img else { continue };
            imgs.push(img)
        }

        let bgcol = if meta.contains_key("bgcol") {
            meta["bgcol"].as_str().unwrap_or("000000")
        } else {
            "000000"
        };
        let bgcol = u32::from_str_radix(bgcol, 16);
        let Ok(bgcol) = bgcol else { continue };

        let talkmode = if meta.contains_key("talkmode") {
            meta["talkmode"].as_str().unwrap_or("000000")
        } else {
            "none"
        };
        let talkmode = loop {
            if talkmode.starts_with("moveup:") {
                let val = talkmode.replace("moveup:", "");
                let Ok(val) = val.parse() else { continue };
                break TalkMode::MoveUp(val)
            }
            if talkmode.starts_with("shake:") {
                let val = talkmode.replace("shake:", "");
                let Ok(val) = val.parse() else { continue };
                break TalkMode::Shake(val)
            }
            break TalkMode::None
        };

        let idlesize = if meta.contains_key("idlesize") {
            let s = meta["idlesize"].as_str().unwrap();
            let mut zee = s.split("x");
            (
                zee.next().unwrap().parse().unwrap(),
                zee.next().unwrap().parse().unwrap(),
            )
        } else {
            (
                imgs[0].size().x,
                imgs[0].size().y,
            )
        };

        let speaksize = if meta.contains_key("speaksize") {
            let s = meta["speaksize"].as_str().unwrap();
            let mut zee = s.split("x");
            (
                zee.next().unwrap().parse().unwrap(),
                zee.next().unwrap().parse().unwrap(),
            )
        } else {
            (
                imgs[1].size().x,
                imgs[1].size().y,
            )
        };

        let avatar = Avatar {
            idle: imgs[0].clone(),
            idlesize,
            speak: imgs[1].clone(),
            speaksize,
            bgcol: Color::from_hex(bgcol),
            talkmode,
        };

        avatars.insert(dir.to_str().expect("should work").to_string(),avatar);
    }
    let mut working_paths: Vec<String> = avatars.keys().cloned().collect();
    working_paths.sort();

    let mut state = State::Select(0, 0.);

    let logo = my_load_texture("misc_assets/logo.png".into(), &mut textures, FilterMode::Nearest).await.expect("expect preshipped assets");

    let settings = Settings {
        thresh: 0.02,
        coyote_time: 0.05,
        move_mod: 2.,
    };

    let mut coyote_time = 0.;

    // let mut move_data: Vec<f32> = vec![];

    loop {
        let dt = get_frame_time();
        match &mut state {
            State::Select(ind, timer) => {
                clear_background(Color::from_hex(0x24273a));

                *timer += dt;

                if is_key_pressed(KeyCode::Z) {
                    state = State::Main(working_paths[*ind].clone());
                    continue;
                }

                if is_key_pressed(KeyCode::Down) {
                    if *ind == working_paths.len() - 1 {
                        *ind = 0
                    } else {
                        *ind += 1
                    }
                }
                if is_key_pressed(KeyCode::Up) {
                    if *ind == 0 {
                        *ind = working_paths.len() - 1
                    } else {
                        *ind -= 1
                    }
                }

                for (i, path) in working_paths.iter().enumerate() {
                    let string = if i == *ind {
                        format!("{}: {} <-", i, path.replace("\\", "/").replace("avatars/", ""))
                    } else {
                        format!("{}: {}", i, path.replace("\\", "/").replace("avatars/", ""))
                    };
                    draw_text(&string, 4., (6 + i) as f32 * 32. + 4., 32., text_col);
                }
                draw_text("arrows + z to select", 4., 160. + 4., 32., text_col);
                let t = *timer;
                for (i, col) in [
                    0xed8796, 0xee9900, 0xf5a97f, 0xeed49f, 0xa6da95, 0x8bd5ca, 0x91d7e3, 0x7dc4e4, 0x8aadf4, 0xb7bdf8, 0xffffff
                ].iter().enumerate() {
                    let toff = t + (i as f32) * -0.1;
                    if toff < 0. { continue; }
                    let toff = toff * 1.25;
                    let x = 512. + 32. * toff.sin() / toff + 20. / toff;
                    let y= 16. + 16. * (0.7 * toff).sin() / toff;
                    draw_texture(&logo, x, y, Color::from_hex(*col));
                }
            }
            State::Main(ref avatar) => {
                let vol = read_balls();
                let avatar = avatars.get(avatar).expect("if this doesn't work i swear to fucking god");
                clear_background(avatar.bgcol);
                let mut should_speak = vol > settings.thresh;
                if vol > settings.thresh {
                    coyote_time = 0.;
                } else if !should_speak && coyote_time < settings.coyote_time {
                    should_speak = true;
                    coyote_time += dt;
                }
                if should_speak {
                    let center_x = (1280. - avatar.speaksize.0) / 2.;
                    let center_y = (720. - avatar.speaksize.1) / 2.;
                    match avatar.talkmode {
                        TalkMode::Shake(mag) => {
                            let bounce_mag = settings.move_mod * vol * mag;
                            let off_x = rand::gen_range(-bounce_mag, bounce_mag);
                            let off_y = rand::gen_range(-bounce_mag, bounce_mag);
                            draw_texture_ex(&avatar.speak, center_x + off_x, center_y + off_y, WHITE, DrawTextureParams {
                                dest_size: Some(avatar.speaksize.into()),
                                ..Default::default()
                            });
                        }
                        TalkMode::MoveUp(mag) => {
                            let bounce_mag = settings.move_mod * vol * mag;
                            draw_texture_ex(&avatar.speak, center_x, center_y - bounce_mag, WHITE, DrawTextureParams {
                                dest_size: Some(avatar.speaksize.into()),
                                ..Default::default()
                            });
                        }
                        _ => {
                            draw_texture_ex(&avatar.speak, center_x, center_y, WHITE, DrawTextureParams {
                                dest_size: Some(avatar.speaksize.into()),
                                ..Default::default()
                            });
                        } 
                    }
                } else {
                    let center_x = (1280. - avatar.idlesize.0) / 2.;
                    let center_y = (720. - avatar.idlesize.1) / 2.;
                    draw_texture_ex(&avatar.idle, center_x, center_y, WHITE, DrawTextureParams {
                        dest_size: Some(avatar.idlesize.into()),
                        ..Default::default()
                    });
                }
                draw_rectangle(0., 720. - 16., 1280., 16., Color::from_hex(0x222222));
                draw_rectangle(0., 720. - 16., 1280. * settings.thresh, 16., GRAY);
                draw_rectangle(0., 720. - 16., 1280. * vol, 16., RED);
            }
        }
        next_frame().await;
    }
}
