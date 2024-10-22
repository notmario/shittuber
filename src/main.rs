use std::time::Duration;
use macroquad::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::HashMap;
use std::fs;
use toml::Table;

const BALLS_SIZE: usize = 2;

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
    Select(f32),
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

const FONT_KERN: [i32; 96] = [
    4, 3, 2, 0, 0, 0, 0, 3, 2, 2, 1, 1, 3, 1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 1,
    2, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
];
const FONT_Y_OFF: [i32; 96] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
];

fn draw_text_cool(tx: &Texture2D, t: &str, x: i32, y: i32, c: Color, scale: i32) {
    if scale <= 0 { return }
    let mut back_off = 0;
    for (i, ch) in t.chars().enumerate() {
        let ind = ch as u32;
        let ind = if ind >= 32 && ind <= 127 {
            ind - 32
        } else {
            95
        };
        let (sx, sy) = (ind % 16, ind / 16);

        let kern = FONT_KERN[ind as usize];
        let yoff = FONT_Y_OFF[ind as usize];

        draw_texture_ex(
            &tx,
            (x + (i as i32 * 12 - kern - back_off) * scale) as f32,
            (y + yoff * scale) as f32,
            c,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: 12. * scale as f32,
                    y: 16. * scale as f32,
                }),
                source: Some(Rect {
                    x: sx as f32 * 12.,
                    y: sy as f32 * 16.,
                    w: 12.,
                    h: 16.,
                }),
                ..Default::default()
            },
        );

        back_off += kern * 2;
    }
}

fn draw_text_cool_c(tx: &Texture2D, t: &str, x: i32, y: i32, c: Color, scale: i32) {
    if scale <= 0 { return }
    let mut total_width = 0;
    for ch in t.chars() {
        let ind = ch as u32;
        let ind = if ind >= 32 && ind <= 127 {
            ind - 32
        } else {
            95
        };
        let kern = FONT_KERN[ind as usize];
        total_width += 12 - kern * 2;
    }

    draw_text_cool(tx, t, x - total_width * scale / 2, y, c, scale);
}

fn draw_text_cool_l(tx: &Texture2D, t: &str, x: i32, y: i32, c: Color, scale: i32) {
    if scale <= 0 { return }
    let mut total_width = 0;
    for ch in t.chars() {
        let ind = ch as u32;
        let ind = if ind >= 32 && ind <= 127 {
            ind - 32
        } else {
            95
        };
        let kern = FONT_KERN[ind as usize];
        total_width += 12 - kern * 2;
    }

    draw_text_cool(tx, t, x - total_width * scale, y, c, scale);
}

fn draw_multiline(tx: &Texture2D, t: &str, x: i32, y: i32, w: i32, c: Color, scale: i32) {
    if scale <= 0 { return }
    let mut c_line_width = 0;
    let mut lines: Vec<usize> = vec![0];
    let mut line_widths = vec![0];
    for word in t.split(" ") {
        let mut my_width = 0;
        for ch in word.chars() {
            let ind = ch as u32;
            let ind = if ind >= 32 && ind <= 127 {
                ind - 32
            } else {
                95
            };
            // let (sx, sy) = (ind % 16, ind / 16);

            let kern = FONT_KERN[ind as usize];
            my_width += 12 - kern * 2;
        }
        if c_line_width + my_width < w / scale {
            c_line_width += my_width + 4;
            *line_widths.last_mut().expect("should have last") += my_width + 4;
            *lines.last_mut().expect("should have last") += word.len() + 1
        } else {
            *line_widths.last_mut().expect("should have last") -= 4;
            c_line_width = my_width + 4;
            line_widths.push(my_width + 4);
            lines.push(word.len() + 1);
        }
    }
    *line_widths.last_mut().expect("should have last") -= 4;
    let mut back_off = 0;
    let mut current_line = 0;
    let mut ind_off = 0;
    for (i, ch) in t.chars().enumerate() {
        if i - ind_off >= lines[current_line] {
            back_off = 0;
            ind_off += lines[current_line];
            current_line += 1;
        }
        let ind = ch as u32;
        let ind = if ind >= 32 && ind <= 127 {
            ind - 32
        } else {
            95
        };
        let (sx, sy) = (ind % 16, ind / 16);

        let kern = FONT_KERN[ind as usize];
        let yoff = FONT_Y_OFF[ind as usize];

        draw_texture_ex(
            &tx,
            (x + ((i - ind_off) as i32 * 12 - kern - back_off - line_widths[current_line] / 2) * scale + w / 2) as f32,
            (y + (yoff + current_line as i32 * 18 - lines.len() as i32 * 9) * scale) as f32,
            c,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: 12. * scale as f32,
                    y: 16. * scale as f32,
                }),
                source: Some(Rect {
                    x: sx as f32 * 12.,
                    y: sy as f32 * 16.,
                    w: 12.,
                    h: 16.,
                }),
                ..Default::default()
            },
        );

        back_off += kern * 2;
    }
}

fn image_button(t: &Texture2D, x: f32, y: f32, w: f32, h: f32) -> bool {
    draw_texture(t, x, y, WHITE);

    let pos = mouse_position();
    if pos.0 > x && pos.0 < x + w &&
    pos.1 > y && pos.1 < y + h {
        draw_texture(t, x, y, Color::from_hex(0xed8796));
        if is_mouse_button_pressed(MouseButton::Left) {
            return true
        }
    }

    false
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    let host = cpal::default_host();
    
    let text_col = Color::from_hex(0xcad3f5);
    let mut textures: HashMap<String, Texture2D> = HashMap::new();
    let font = my_load_texture("misc_assets/letters.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let back_button = my_load_texture("misc_assets/back.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");

    let selected_ind;

    let devs: Vec<cpal::Device> = 
        host.devices().expect("I CANT EVEN")
        .filter(|a| a.supported_input_configs().is_ok_and(|k| k.count() > 0)).collect();

    'outer: loop {
        clear_background(Color::from_hex(0x1e2030));

        draw_text_cool(&font, "CHOOSE YOUR AUDIO DEVICE", 4, 4, text_col, 2);
        draw_text_cool(&font, "if yours doesnt show up SHOUT AT ME LOUDLY", 4, 36, text_col, 2);

        let mouse_pos = mouse_position();

        let off_y = 72.;
        let grid_size = 192.;

        for (i, d) in devs.iter().enumerate() {
            let grid_x = (i % 6) as i32;
            let grid_y = (i / 6) as i32;
            draw_rectangle(8. + grid_size * grid_x as f32, off_y + 4. + grid_size * grid_y as f32, 
                grid_size - 16., grid_size - 16., text_col);
            draw_rectangle(12. + grid_size * grid_x as f32, off_y + 8. + grid_size * grid_y as f32, 
                grid_size - 24., grid_size - 24., Color::from_hex(0x1e2030));
            if mouse_pos.0 > grid_size * grid_x as f32 && mouse_pos.0 < grid_size * (grid_x + 1) as f32 &&
                mouse_pos.1 > off_y + grid_size * grid_y as f32 && mouse_pos.1 < off_y + grid_size * (grid_y + 1) as f32 {
                draw_rectangle(12. + grid_size * grid_x as f32, off_y + 8. + grid_size * grid_y as f32, 
                    grid_size - 24., grid_size - 24., Color::from_hex(0x494d64));
                if is_mouse_button_pressed(MouseButton::Left) {
                    selected_ind = i;
                    next_frame().await;
                    break 'outer
                }
            }
            // let string = if i == selected_ind {
            //     format!("=>  {}: {}", i, d.name().unwrap_or("MYSTERY SHIT".into()))
            // } else {
            //     format!("        {}: {}", i, d.name().unwrap_or("MYSTERY SHIT".into()))
            // };
            // draw_text_cool(&font, &string, 4, (3 + i) as i32 * 32, text_col, 2);
            let rgs = grid_size as i32;
            draw_multiline(&font,&d.name().unwrap_or("MYSTERY SHIT".into()), 16 + rgs * grid_x, 166 + rgs * grid_y, rgs - 32, text_col, 1);
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
            let sum: f32 = data.iter().map(|a| a.abs()).sum();
            let avg = sum / data.len() as f32;
            add_to_balls(avg);
        },
        move |_| {
            // uh?
        },
        Some(Duration::from_millis(1000)) // None=blocking, Some(Duration)=timeout
    ).expect("this should exist");
    stream.play().unwrap();

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

    let mut state = State::Select(0.);

    let logo = my_load_texture("misc_assets/logo.png".into(), &mut textures, FilterMode::Nearest).await.expect("expect preshipped assets");

    let settings = Settings {
        thresh: 0.02,
        coyote_time: 0.05,
        move_mod: 2.,
    };

    let mut coyote_time = 0.;

    // let mut move_data: Vec<f32> = vec![];

    let mut show_ui = true;
    let mut show_ui_thingy_timey = 0.;

    'outer: loop {
        let dt = get_frame_time();
        match &mut state {
            State::Select(timer) => {
                clear_background(Color::from_hex(0x24273a));

                *timer += dt;

                // if is_key_pressed(KeyCode::Z) {
                //     state = State::Main(working_paths[*ind].clone());
                //     continue;
                // }

                // if is_key_pressed(KeyCode::Down) {
                //     if *ind == working_paths.len() - 1 {
                //         *ind = 0
                //     } else {
                //         *ind += 1
                //     }
                // }
                // if is_key_pressed(KeyCode::Up) {
                //     if *ind == 0 {
                //         *ind = working_paths.len() - 1
                //     } else {
                //         *ind -= 1
                //     }
                // }
                
                let mouse_pos = mouse_position();
                let off_y = 160.;
                let grid_size = 128.;
        
                for (i, p) in working_paths.iter().enumerate() {
                    let grid_x = (i % 10) as i32;
                    let grid_y = (i / 10) as i32;
                    draw_rectangle(
                        8. + grid_size * grid_x as f32, 
                        off_y + 4. + grid_size * grid_y as f32, 
                        grid_size - 16., grid_size - 16., text_col
                    );
                    draw_rectangle(
                        12. + grid_size * grid_x as f32, 
                        off_y + 8. + grid_size * grid_y as f32, 
                        grid_size - 24., grid_size - 24., Color::from_hex(0x1e2030));
                    if mouse_pos.0 > grid_size * grid_x as f32 && mouse_pos.0 < grid_size * (grid_x + 1) as f32 &&
                        mouse_pos.1 > off_y + grid_size * grid_y as f32 && mouse_pos.1 < off_y + grid_size * (grid_y + 1) as f32 {
                        draw_rectangle(12. + grid_size * grid_x as f32, off_y + 8. + grid_size * grid_y as f32, grid_size - 24., grid_size - 24., Color::from_hex(0x494d64));
                        if is_mouse_button_pressed(MouseButton::Left) {
                            state = State::Main(working_paths[i].clone());
                            show_ui = true;
                            next_frame().await;
                            continue 'outer
                        }
                    }
                    let t= &avatars[p].speak;
                    draw_texture_ex(t, 32. + grid_size * grid_x as f32, off_y + 24. + grid_size * grid_y as f32, WHITE, DrawTextureParams {
                        dest_size: Some(Vec2 {
                            x: 64., y: 64.
                        }),
                        ..Default::default()
                    });
                    let rgs = grid_size as i32;
                    draw_text_cool_c(&font,&p.replace("\\", "/").replace("avatars/", ""), rgs / 2 + rgs * grid_x, off_y as i32 + rgs - 36 + rgs * grid_y, text_col, 1);
                }
                draw_text_cool(&font, "CHOOSE YOUR THING", 4, 128, text_col, 2);
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
                show_ui_thingy_timey += dt;
                if is_mouse_button_pressed(MouseButton::Left) {
                    if show_ui_thingy_timey < 0.25 {
                        show_ui = !show_ui;
                        show_ui_thingy_timey = 5.;
                    } else {
                        show_ui_thingy_timey = 0.;
                    }
                }
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
                if show_ui {
                    draw_rectangle(0., 720. - 16., 1280., 16., Color::from_hex(0x181926));
                    draw_rectangle(0., 720. - 16., 1280. * settings.thresh, 16., Color::from_hex(0xa5adcb));
                    draw_rectangle(0., 720. - 16., 1280. * vol, 16., Color::from_hex(0xed8796));
                    if image_button(&back_button, 16., 16., 64., 64.) {
                        state = State::Select(0.)
                    }

                    draw_text_cool_l(&font, "DOUBLE CLICK ANYWHERE TO HIDE/SHOW", 1280 - 2, 720 - 48, RED, 2);
                }
            }
        }
        next_frame().await;
    }
}
