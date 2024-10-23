use std::time::Duration;
use macroquad::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::HashMap;
use std::fs;
use toml::Table;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers}, GlobalHotKeyEvent};

mod agony;
use agony::mq_key_to_global_hotkey;

// this is not in actual samples (Duh) but in whatever we get from cpal
// uh maybe make this better(?)
const BUFFER_SIZE: usize = 3;

static mut AUDIO_BUFFER: [f32; BUFFER_SIZE] = [0.;BUFFER_SIZE];
static mut AUDIO_BUFFER_POINTER: usize = 0;

static mut CURRENT_EMOTION: usize = 0;

fn add_to_buffer(data: f32) {
    unsafe {
        AUDIO_BUFFER[AUDIO_BUFFER_POINTER] = data;
        if AUDIO_BUFFER_POINTER < BUFFER_SIZE - 1 {
            AUDIO_BUFFER_POINTER += 1
        } else {
            AUDIO_BUFFER_POINTER = 0
        }
    }
}

fn read_buffer() -> f32 {
    let mut sum = 0.;
    unsafe {
        for dat in AUDIO_BUFFER {
            sum += dat
        }
    }
    sum / BUFFER_SIZE as f32
}

enum State {
    Select(f32),
    Main(String),
    Settings(Option<String>),
}

#[derive(Debug)]
struct Settings {
    thresh: f32,
    coyote_time: f32,
    use_ctrl: bool,
    use_shift: bool,
    use_alt: bool,
    hotkey_1: KeyCode,
    hotkey_2: KeyCode,
    hotkey_3: KeyCode,
    hotkey_4: KeyCode,
    hotkey_5: KeyCode,
    hotkey_6: KeyCode,
    hotkey_7: KeyCode,
    hotkey_8: KeyCode,
    hotkey_9: KeyCode,
}

#[derive(Debug)]
enum TalkMode {
    None,
    MoveUp(f32),
    Shake(f32),
    Jump(f32, f32),
}

#[derive(Debug)]
struct Emotion {
    idle: Texture2D,
    idlesize: (f32, f32),
    speak: Texture2D,
    speaksize: (f32, f32),
    talkmode: TalkMode,
}

struct Avatar {
    emotions: Vec<Emotion>,
    bgcol: Color,
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
        draw_texture(t, x, y, Color::from_hex(0xcad3f5));
        if is_mouse_button_down(MouseButton::Left) {
            draw_texture(t, x, y, Color::from_hex(0xed8796));
        }
        if is_mouse_button_released(MouseButton::Left) {
            return true
        }
    }

    false
}

fn render_guy(avatar: &Avatar, settings: &Settings, vol: f32, coyote_time: &mut f32, dt: f32, data: &mut Vec<f32>) {
    let mut should_speak = vol > settings.thresh;
    if vol > settings.thresh {
        *coyote_time = 0.;
    } else if !should_speak && *coyote_time < settings.coyote_time {
        should_speak = true;
        *coyote_time += dt;
    }
    let current_emotion = unsafe {CURRENT_EMOTION};
    if let TalkMode::Jump(..) = avatar.emotions[current_emotion].talkmode {
        if data.len() != 3 { *data = vec![0., 0., 0.] }
    } else {
        *data = vec![];
    }
    if should_speak {
        let center_x = (1280. - avatar.emotions[current_emotion].speaksize.0) / 2.;
        let center_y = (720. - avatar.emotions[current_emotion].speaksize.1) / 2.;
        match avatar.emotions[current_emotion].talkmode {
            TalkMode::Jump(mag, grav) => {
                if data[1] != 0. {
                    data[1] += data[0] / 2. * dt;
                    data[0] += dt * grav;
                    data[1] += data[0] / 2. * dt;
                    if data[1] > 0. {
                        data[1] = 0.;
                        data[0] = 0.;
                    }
                }
                if data[2] == 0. {
                    data[2] = 1.;
                    data[1] += -1.;
                    // v^2 = 2as
                    // v = sqrt(2as)
                    data[0] += -(mag * grav * std::f32::consts::SQRT_2).sqrt();
                }
                draw_texture_ex(&avatar.emotions[current_emotion].speak, center_x, center_y + data[1], WHITE, DrawTextureParams {
                    dest_size: Some(avatar.emotions[current_emotion].speaksize.into()),
                    ..Default::default()
                });
            }
            TalkMode::Shake(mag) => {
                let bounce_mag = vol.sqrt() * mag;
                let off_x = rand::gen_range(-bounce_mag, bounce_mag);
                let off_y = rand::gen_range(-bounce_mag, bounce_mag);
                draw_texture_ex(&avatar.emotions[current_emotion].speak, center_x + off_x, center_y + off_y, WHITE, DrawTextureParams {
                    dest_size: Some(avatar.emotions[current_emotion].speaksize.into()),
                    ..Default::default()
                });
            }
            TalkMode::MoveUp(mag) => {
                let bounce_mag = vol.sqrt() * mag;
                draw_texture_ex(&avatar.emotions[current_emotion].speak, center_x, center_y - bounce_mag, WHITE, DrawTextureParams {
                    dest_size: Some(avatar.emotions[current_emotion].speaksize.into()),
                    ..Default::default()
                });
            }
            _ => {
                draw_texture_ex(&avatar.emotions[current_emotion].speak, center_x, center_y, WHITE, DrawTextureParams {
                    dest_size: Some(avatar.emotions[current_emotion].speaksize.into()),
                    ..Default::default()
                });
            } 
        }
    } else {
        let center_x = (1280. - avatar.emotions[current_emotion].idlesize.0) / 2.;
        let center_y = (720. - avatar.emotions[current_emotion].idlesize.1) / 2.;
        match avatar.emotions[current_emotion].talkmode {
            TalkMode::Jump(..) => {
                if data[1] != 0. {
                    data[1] += data[0] / 2. * dt;
                    data[0] += dt * 512.;
                    data[1] += data[0] / 2. * dt;
                    if data[1] > 0. {
                        data[1] = 0.;
                        data[0] = 0.;
                    }
                }
                data[2] = 0.;
                draw_texture_ex(&avatar.emotions[current_emotion].idle, center_x, center_y + data[1], WHITE, DrawTextureParams {
                    dest_size: Some(avatar.emotions[current_emotion].idlesize.into()),
                    ..Default::default()
                });
            }
            _ => {
                draw_texture_ex(&avatar.emotions[current_emotion].idle, center_x, center_y, WHITE, DrawTextureParams {
                    dest_size: Some(avatar.emotions[current_emotion].idlesize.into()),
                    ..Default::default()
                });
            }
        }
    }
}

fn write_settings(s: &Settings) {
    let mut st = String::new();
    st.push_str(&s.thresh.to_string());
    st.push('\n');
    st.push_str(&s.coyote_time.to_string());
    st.push('\n');
    let boole = |b| if b { "true" } else { "false" };
    st.push_str(boole(s.use_ctrl));
    st.push('\n');
    st.push_str(boole(s.use_shift));
    st.push('\n');
    st.push_str(boole(s.use_alt));
    st.push('\n');
    // this one is fine actually
    let wizard_shit = |k: KeyCode| -> u16 {unsafe {std::mem::transmute(k)}};
    st.push_str(&wizard_shit(s.hotkey_1).to_string());
    st.push('\n');
    st.push_str(&wizard_shit(s.hotkey_2).to_string());
    st.push('\n');
    st.push_str(&wizard_shit(s.hotkey_3).to_string());
    st.push('\n');
    st.push_str(&wizard_shit(s.hotkey_4).to_string());
    st.push('\n');
    st.push_str(&wizard_shit(s.hotkey_5).to_string());
    st.push('\n');
    st.push_str(&wizard_shit(s.hotkey_6).to_string());
    st.push('\n');
    st.push_str(&wizard_shit(s.hotkey_7).to_string());
    st.push('\n');
    st.push_str(&wizard_shit(s.hotkey_8).to_string());
    st.push('\n');
    st.push_str(&wizard_shit(s.hotkey_9).to_string());

    let _ = std::fs::write("settings", &st);
}

fn reset_hotkeys(s: &Settings, manager: &GlobalHotKeyManager, prev_hotkeys: &[HotKey]) -> Vec<HotKey> {
    let _ = manager.unregister_all(prev_hotkeys);
    let mut new_hotkeys = vec![];
    let mut mods: Modifiers = Modifiers::empty();
    if s.use_ctrl { mods |= Modifiers::CONTROL };
    if s.use_alt { mods |= Modifiers::ALT };
    if s.use_shift { mods |= Modifiers::SHIFT };
    for k in [
        s.hotkey_1,s.hotkey_2,s.hotkey_3,s.hotkey_4,s.hotkey_5,s.hotkey_6,s.hotkey_7,s.hotkey_8,s.hotkey_9,
    ] {
        let real_key = mq_key_to_global_hotkey(k);
        let hotkey = HotKey::new(Some(mods), real_key);
        let _ = manager.register(hotkey);
        new_hotkeys.push(hotkey);
    }
    new_hotkeys
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    let host = cpal::default_host();
    
    let text_col = Color::from_hex(0xcad3f5);
    let mut textures: HashMap<String, Texture2D> = HashMap::new();
    let font = my_load_texture("misc_assets/letters.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let back_button = my_load_texture("misc_assets/back.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let settings_button = my_load_texture("misc_assets/settings.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let emotion_button = my_load_texture("misc_assets/emotion.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let arrow = my_load_texture("misc_assets/arrow.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let idle_icon = my_load_texture("misc_assets/idle.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let speak_icon = my_load_texture("misc_assets/speak.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let tick_on = my_load_texture("misc_assets/tickboxticked.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let tick_off = my_load_texture("misc_assets/tickboxempty.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");
    let hkeybox = my_load_texture("misc_assets/hkeybox.png".into(), &mut textures, FilterMode::Nearest).await.expect("should have");

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
                if is_mouse_button_down(MouseButton::Left) {
                    draw_rectangle(12. + grid_size * grid_x as f32, off_y + 8. + grid_size * grid_y as f32, 
                        grid_size - 24., grid_size - 24., Color::from_hex(0x181926));
                }
                if is_mouse_button_released(MouseButton::Left) {
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
            add_to_buffer(avg);
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
        
        if let Ok(_) = std::fs::read_to_string(dir.join("disabled")) {
            continue
        }

        let meta = std::fs::read_to_string(dir.join("meta.toml")).unwrap_or("".into());
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

        let bgcol = if meta.contains_key("bgcol") {
            meta["bgcol"].as_str().unwrap_or("00FF00")
        } else {
            "00FF00"
        };
        let bgcol = u32::from_str_radix(bgcol, 16);
        let Ok(bgcol) = bgcol else { continue };

        let k = meta.get("emotions");
        println!("{:?}", k);
        let emotions: Vec<_> = if let Some(emotions) = k {
            emotions.as_array().expect("should be array").iter().map(|emotion| {
                emotion.as_table().expect("should be table")
            }).cloned().collect()
        } else {
            vec![toml::Table::new()]
        };

        let mut emotions_parsed = vec![];

        for (i, emotion) in emotions.iter().enumerate() {
            println!("{:?}", emotion);
            let thingy = if i == 0 { "".into() } else { (i+1).to_string() };
            let paths = [format!("idle{}.png",thingy), format!("speak{}.png",thingy)];
            let mut imgs = vec![];
            for path in paths {
                let img = my_load_texture(dir.join(path).to_str().expect("should exist").into(), &mut textures, filtermode).await;
                let Ok(img) = img else { continue };
                imgs.push(img)
            }
    
            let talkmode = if emotion.contains_key("talkmode") {
                emotion["talkmode"].as_str().unwrap_or("none")
            } else {
                "none"
            };
            let talkmode = loop {
                if talkmode.starts_with("moveup:") {
                    let val = talkmode.replace("moveup:", "");
                    let Ok(val) = val.parse() else { continue };
                    break TalkMode::MoveUp(val)
                }
                if talkmode.starts_with("jump:") {
                    let val = talkmode.replace("jump:", "");
                    let mut vals = val.split(":");
                    let Ok(mag) = vals.next().expect("should have").parse() else { continue };
                    let Ok(grav) = vals.next().expect("should have").parse() else { continue };
                    break TalkMode::Jump(mag, grav)
                }
                if talkmode.starts_with("shake:") {
                    let val = talkmode.replace("shake:", "");
                    let Ok(val) = val.parse() else { continue };
                    break TalkMode::Shake(val)
                }
                break TalkMode::None
            };
    
            let idlesize = if emotion.contains_key("idlesize") {
                let s = emotion["idlesize"].as_str().unwrap();
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
    
            let speaksize = if emotion.contains_key("speaksize") {
                let s = emotion["speaksize"].as_str().unwrap();
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

            emotions_parsed.push(
                Emotion {
                    idle: imgs[0].clone(),
                    idlesize,
                    speak: imgs[1].clone(),
                    speaksize,
                    talkmode,
                }
            )
        }

        println!("{:?}", emotions_parsed);

        let avatar = Avatar {
            emotions: emotions_parsed,
            bgcol: Color::from_hex(bgcol),
        };

        avatars.insert(dir.to_str().expect("should work").to_string(),avatar);
    }
    let mut working_paths: Vec<String> = avatars.keys().cloned().collect();
    working_paths.sort();

    let logo = my_load_texture("misc_assets/logo.png".into(), &mut textures, FilterMode::Nearest).await.expect("expect preshipped assets");

    let preexisting_settings = {
        if let Ok(file) = std::fs::read_to_string("settings") {
            let lines: Box<[&str]> = file.split("\n").collect();
            if lines.len() < 14 { 
                None
            } else {
                // Um. This is probably a Shit Idea but I Do Not Care at This Point
                // if the User Fucks It Up That is Their Fucking Fault
                let z = |k:u16| {
                    unsafe {
                        std::mem::transmute(k)
                    }
                };
                Some(Settings {
                    thresh: lines[0].parse().unwrap(),
                    coyote_time: lines[1].parse().unwrap(),
                    use_ctrl: lines[2] == "true",
                    use_shift: lines[3] == "true",
                    use_alt: lines[4] == "true",
                    hotkey_1: z(lines[5].parse().unwrap()),
                    hotkey_2: z(lines[6].parse().unwrap()),
                    hotkey_3: z(lines[7].parse().unwrap()),
                    hotkey_4: z(lines[8].parse().unwrap()),
                    hotkey_5: z(lines[9].parse().unwrap()),
                    hotkey_6: z(lines[10].parse().unwrap()),
                    hotkey_7: z(lines[11].parse().unwrap()),
                    hotkey_8: z(lines[12].parse().unwrap()),
                    hotkey_9: z(lines[13].parse().unwrap()),
                })
            }
        } else {
            None
        }
    };
    
    let mut state = if (&preexisting_settings).is_some() {
        State::Select(0.)
    } else {
        State::Settings(None)
    };

    let mut settings = preexisting_settings.unwrap_or(Settings {
        thresh: 0.5,
        coyote_time: 0.05,
        use_ctrl: false,
        use_shift: true,
        use_alt: true,
        hotkey_1: KeyCode::Key1,
        hotkey_2: KeyCode::Key2,
        hotkey_3: KeyCode::Key3,
        hotkey_4: KeyCode::Key4,
        hotkey_5: KeyCode::Key5,
        hotkey_6: KeyCode::Key6,
        hotkey_7: KeyCode::Key7,
        hotkey_8: KeyCode::Key8,
        hotkey_9: KeyCode::Key9,
    });

    println!("{:?}", settings);

    let mut coyote_time = 0.;

    let mut move_data: Vec<f32> = vec![];

    let mut show_ui = true;
    let mut show_ui_thingy_timey = 0.;

    let mut listening_for_key: Option<usize> = None;

    let manager = GlobalHotKeyManager::new().unwrap();
    let mut hotkeys = reset_hotkeys(&settings, &manager, &[]);

    'outer: loop {
        let dt = get_frame_time();
        match &mut state {
            State::Select(timer) => {
                clear_background(Color::from_hex(0x24273a));

                *timer += dt;

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
                        if is_mouse_button_down(MouseButton::Left) {
                            draw_rectangle(12. + grid_size * grid_x as f32, off_y + 8. + grid_size * grid_y as f32, grid_size - 24., grid_size - 24., Color::from_hex(0x181926));
                        }
                        if is_mouse_button_released(MouseButton::Left) {
                            state = State::Main(working_paths[i].clone());
                            unsafe { CURRENT_EMOTION = 0 };
                            show_ui = true;
                            next_frame().await;
                            continue 'outer
                        }
                    }
                    let t= &avatars[p].emotions[unsafe { CURRENT_EMOTION }].speak;
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
                if image_button(&settings_button, 1200., 16., 64., 64.) {
                    state = State::Settings(None)
                }
            }
            State::Main(ref avatar) => {
                show_ui_thingy_timey += dt;

                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.state == global_hotkey::HotKeyState::Pressed {
                        let avatar = avatars.get(avatar).expect("if this doesn't work i swear to fucking god");
                        for (i, hk) in hotkeys.iter().enumerate() {
                            if i >= avatar.emotions.len() { break }
                            if hk.id == event.id {
                                unsafe { CURRENT_EMOTION = i }
                            }
                        }
                    }
                }
                
                if is_mouse_button_pressed(MouseButton::Left) {
                    if show_ui_thingy_timey < 0.25 {
                        show_ui = !show_ui;
                        show_ui_thingy_timey = 5.;
                    } else {
                        show_ui_thingy_timey = 0.;
                    }
                }
                let vol = read_buffer();
                {
                    let avatar = avatars.get(avatar).expect("if this doesn't work i swear to fucking god");
                    clear_background(avatar.bgcol);
                    render_guy(&avatar, &settings, vol, &mut coyote_time, dt, &mut move_data);
                }
                if show_ui {
                    draw_rectangle(0., 720. - 16., 1280., 16., Color::from_hex(0x181926));
                    draw_rectangle(0., 720. - 16., 1280. * settings.thresh.sqrt(), 16., Color::from_hex(0xa5adcb));
                    draw_rectangle(0., 720. - 16., 1280. * vol.sqrt(), 16., Color::from_hex(0xed8796));
                    {
                        let avatar = avatars.get(avatar).expect("if this doesn't work i swear to fucking god");
                        for (i, _) in avatar.emotions.iter().enumerate() {
                            if image_button(&emotion_button, 1200., 16. + 80. * (1 + i) as f32, 64., 64.) {
                                unsafe { CURRENT_EMOTION = i }
                            }
                            draw_text_cool_c(&font, &(i + 1).to_string(), 1232, 16 + 80 * (1 + i) as i32, Color::from_hex(0xed8796), 4);
                        }
                    }

                    if image_button(&back_button, 16., 16., 64., 64.) {
                        state = State::Select(0.)
                    } else if image_button(&settings_button, 1200., 16., 64., 64.) {
                        state = State::Settings(Some(avatar.clone()))
                    }

                    draw_text_cool_l(&font, "DOUBLE CLICK ANYWHERE TO HIDE/SHOW", 1280 - 2, 720 - 48, RED, 2);
                }
            }
            State::Settings(ref avatar) => {
                if let Some(ind) = listening_for_key {
                    clear_background(Color::from_hex(0x000000));
                    draw_text_cool_c(&font, &format!("LISTENING FOR KEY {}", ind), 640, 320, text_col, 5);
                    for k in get_keys_pressed() {
                        if k == KeyCode::Escape {
                            listening_for_key = None;
                            next_frame().await;
                            continue
                        }
                        if k != KeyCode::LeftControl && k != KeyCode::RightControl && k != KeyCode::LeftAlt && k != KeyCode::RightAlt && k != KeyCode::LeftShift && k != KeyCode::RightShift {
                            listening_for_key = None;
                            next_frame().await;
                            match ind {
                                1 => settings.hotkey_1 = k,
                                2 => settings.hotkey_2 = k,
                                3 => settings.hotkey_3 = k,
                                4 => settings.hotkey_4 = k,
                                5 => settings.hotkey_5 = k,
                                6 => settings.hotkey_6 = k,
                                7 => settings.hotkey_7 = k,
                                8 => settings.hotkey_8 = k,
                                9 => settings.hotkey_9 = k,
                                _ => ()
                            }
                            continue
                        }
                    }
                    next_frame().await;
                    continue
                }
                let vol = read_buffer();
                let mouse_pos = mouse_position();
                if let Some(avatar) = avatar {
                    let avatar = avatars.get(avatar).expect("if this doesn't work i swear to fucking god");
                    clear_background(avatar.bgcol);
                    render_guy(&avatar, &settings, vol, &mut coyote_time, dt, &mut move_data);
                    draw_rectangle(0., 0., 1280., 720., Color::from_rgba(30, 32, 48, 192));
                } else {
                    clear_background(Color::from_hex(0x1e2030));
                    let mut should_speak = vol > settings.thresh;
                    if vol > settings.thresh {
                        coyote_time = 0.;
                    } else if !should_speak && coyote_time < settings.coyote_time {
                        should_speak = true;
                        coyote_time += dt;
                    }
                    if should_speak {
                        draw_texture(&speak_icon, 1152., 592., if coyote_time != 0. {
                            RED
                        } else {
                            WHITE
                        });
                    } else {
                        draw_texture(&idle_icon, 1152., 592., WHITE);
                    }
                }
                draw_text_cool_c(&font, "SETTINGS", 640, 24, text_col, 3);
                draw_rectangle(32., 32., 32., 656., Color::from_hex(0x24273a));
                draw_rectangle(32., 32. + 656. * (1. - settings.thresh.sqrt()), 32., 656. * settings.thresh.sqrt(), Color::from_hex(0xa5adcb));
                draw_rectangle(32., 32. + 656. * (1. - vol.sqrt()), 32., 656. * vol.sqrt(), Color::from_hex(0xed8796));
                draw_rectangle(64., 32., 32., 656., Color::from_hex(0x181926));
                draw_texture(&arrow, 64., 16. + 656. * (1. - settings.thresh.sqrt()), WHITE);
                if mouse_pos.0 > 16. && mouse_pos.0 < 112. && mouse_pos.1 > 16. && mouse_pos.1 < 704. {
                    if is_mouse_button_down(MouseButton::Left) {
                        settings.thresh = (1.-(mouse_pos.1 - 32.) / 656.).clamp(0., 1.).powi(2);
                    }
                }
                draw_text_cool(&font, "ADJUST MICROPHONE THRESHOLD", 104, 668, text_col, 1);
                draw_text_cool_c(&font, "adjust release time", 768, 112, text_col, 2);
                draw_text_cool_c(&font, "this is the amount of time the mic stays on before turning off", 768, 144, text_col, 1);
                draw_text_cool_c(&font, "useful if your mic is noisy, or for sounds like 's' etc.", 768, 160, text_col, 1);
                
                draw_rectangle(384., 188., 768., 24., Color::from_hex(0x24273a));
                draw_rectangle(384., 188., 768. * settings.coyote_time * 2., 24., Color::from_hex(0xed8796));
                draw_text_cool_c(&font, &format!("currently {:.2}s", settings.coyote_time), 768, 220, text_col, 1);
                if mouse_pos.0 > 368. && mouse_pos.0 < 1168. && mouse_pos.1 > 172. && mouse_pos.1 < 228. {
                    if is_mouse_button_down(MouseButton::Left) {
                        settings.coyote_time = ((mouse_pos.0 - 384.) / 768. / 2.).max(0.);
                    }
                }
                
                draw_text_cool_c(&font, "HOT KEYS", 640, 300, text_col, 2);
                draw_text_cool_c(&font, "to change Your Emotion", 640, 336, text_col, 1);
                draw_text_cool_c(&font, "ctrl", 400, 360, text_col, 1);
                if image_button(if settings.use_ctrl { &tick_on } else { &tick_off }, 384., 380., 32., 32.) {
                    settings.use_ctrl = !settings.use_ctrl;
                }
                draw_text_cool_c(&font, "alt", 400, 420, text_col, 1);
                if image_button(if settings.use_alt { &tick_on } else { &tick_off }, 384., 440., 32., 32.) {
                    settings.use_alt = !settings.use_alt;
                }
                draw_text_cool_c(&font, "shift", 400, 480, text_col, 1);
                if image_button(if settings.use_shift { &tick_on } else { &tick_off }, 384., 500., 32., 32.) {
                    settings.use_shift = !settings.use_shift;
                }
                let offy = 380.;
                for (i, val) in [
                    settings.hotkey_1,
                    settings.hotkey_2,
                    settings.hotkey_3,
                    settings.hotkey_4,
                    settings.hotkey_5,
                    settings.hotkey_6,
                    settings.hotkey_7,
                    settings.hotkey_8,
                    settings.hotkey_9,
                ].iter().enumerate() {
                    let x = i % 3;
                    let y= i / 3;
                    if image_button(&hkeybox, 512. + x as f32 * 128., offy + y as f32 * 52., 96., 32.) {
                        listening_for_key = Some(i+1)
                    }
                    draw_text_cool_c(&font, &format!("#{:?}", i+1), 560 + x as i32 * 128, offy as i32 - 18 + y as i32 * 52, text_col, 1);
                    draw_text_cool_c(&font, &format!("{:?}", val), 560 + x as i32 * 128, offy as i32 + 8 + y as i32 * 52, text_col, 1);
                }
                
                if image_button(&back_button, 1200., 16., 64., 64.) {
                    write_settings(&settings);
                    hotkeys = reset_hotkeys(&settings, &manager, &hotkeys);
                    if let Some(avatar) = avatar {
                        state = State::Main(avatar.clone())
                    } else {
                        state = State::Select(0.)
                    }
                }
            }
        }
        next_frame().await;
    }
}
