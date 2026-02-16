use clap::{Parser, ValueEnum};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, PrintStyledContent, Stylize},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use rand::Rng;
use std::cmp::min;
use std::io::{stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ColorSetName {
    Determination,
    City,
    #[value(name = "2077")]
    C2077,
    Thermography,
}

#[derive(Clone, Debug)]
struct ColorSet {
    colors: Vec<Color>,
}

impl ColorSet {
    fn from_name(name: ColorSetName) -> Self {
        match name {
            ColorSetName::Determination => Self::from_hex(&["#39c4b6", "#fee801", "#6300ff"]),
            ColorSetName::City => Self::from_hex(&["#ff0677", "#0051ff", "#8900ff"]),
            ColorSetName::C2077 => {
                Self::from_hex(&["#c5003c", "#880425", "#f3e600", "#55ead4"])
            }
            ColorSetName::Thermography => {
                Self::from_hex(&["#ff004a", "#ffcc3d", "#ff5631", "#ad00ff"])
            }
        }
    }

    fn from_hex(hexes: &[&str]) -> Self {
        let mut colors = Vec::new();
        for h in hexes {
            if let Some(c) = hex_to_color(h) {
                colors.push(c);
            }
        }
        if colors.is_empty() {
            colors.push(Color::Green);
        }
        Self { colors }
    }

    fn gradient_color(&self, t: f32) -> Color {
        // t in [0,1], map über Palette
        if self.colors.len() == 1 {
            return self.colors[0];
        }
        let n = self.colors.len();
        let scaled = t.clamp(0.0, 1.0) * (n as f32 - 1.0);
        let i = scaled.floor() as usize;
        let j = min(i + 1, n - 1);
        let local_t = scaled - i as f32;

        blend_color(self.colors[i], self.colors[j], local_t)
    }
}

fn hex_to_color(hex: &str) -> Option<Color> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some(Color::Rgb { r, g, b })
}

fn blend_color(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = color_to_rgb(a);
    let (br, bg, bb) = color_to_rgb(b);
    let t = t.clamp(0.0, 1.0);
    let r = (ar as f32 + (br as f32 - ar as f32) * t) as u8;
    let g = (ag as f32 + (bg as f32 - ag as f32) * t) as u8;
    let b = (ab as f32 + (bb as f32 - ab as f32) * t) as u8;
    Color::Rgb { r, g, b }
}

fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::DarkGrey => (80, 80, 80),
        Color::Grey => (128, 128, 128),
        Color::White => (255, 255, 255),
        Color::Red => (255, 0, 0),
        Color::DarkRed => (128, 0, 0),
        Color::Green => (0, 255, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::Blue => (0, 0, 255),
        Color::DarkBlue => (0, 0, 128),
        Color::Yellow => (255, 255, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::Magenta => (255, 0, 255),
        Color::DarkMagenta => (128, 0, 128),
        Color::Cyan => (0, 255, 255),
        Color::DarkCyan => (0, 128, 128),
        _ => (0, 255, 0),
    }
}

#[derive(Parser, Debug)]
#[command(name = "matrix", version, about = "Retro-futuristischer Matrix-Digital-Rain in Rust")]
struct Args {
    /// Ziel-String, in dem die fallenden Zeichen „einrasten“
    #[arg(short, long, default_value = "Hallo Welt!")]
    string: String,

    /// Farbset: determination, city, 2077, thermography
    #[arg(short, long, value_enum)]
    colorset: Option<ColorSetName>,

    /// Liste der verfügbaren Farbsets anzeigen und beenden
    #[arg(long, conflicts_with = "colorset")]
    list: bool,
}

#[derive(Clone)]
struct Column {
    x: u16,
    head_y: i16,
    speed: u64,
}

fn random_char(chars: &[char], offset: usize) -> char {
    let mut rng = rand::thread_rng();
    let idx = (rng.gen_range(0..chars.len()) + offset) % chars.len();
    chars[idx]
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if args.list {
        println!("Verfügbare Farbsets:");
        for variant in ColorSetName::value_variants() {
            if let Some(value) = variant.to_possible_value() {
                println!("  {}", value.get_name());
            }
        }
        return Ok(());
    }

    let target = args.string;
    let colorset = ColorSet::from_name(args.colorset.unwrap_or(ColorSetName::Determination));

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    let (width, height) = terminal::size()?;
    let height_i16 = height as i16;

    // Ziel-String mittig unten
    let target_len = target.chars().count() as u16;
    let start_x = if target_len < width {
        (width - target_len) / 2
    } else {
        0
    };
    let target_y = height.saturating_sub(2); // eine Zeile über der letzten

    // Für jedes Zeichen im Ziel-String merken wir, ob es schon „eingeloggt“ ist
    let mut locked_chars: Vec<Option<char>> = vec![None; target_len as usize];

    // Zeichensatz für Regen
    let mut charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    let mut charset_offset: usize = 0;

    // Spalten initialisieren
    let mut rng = rand::thread_rng();
    let mut columns: Vec<Column> = (0..width)
        .map(|x| Column {
            x,
            head_y: rng.gen_range(-20..0),
            speed: rng.gen_range(40..120), // ms pro Schritt
        })
        .collect();

    // Zeittracking pro Spalte
    let mut last_update: Vec<Instant> = columns.iter().map(|_| Instant::now()).collect();

    // Hintergrund schwarz
    stdout
        .execute(terminal::Clear(ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;

    // Hauptloop
    'outer: loop {
        // Eingabe prüfen (q oder ESC beendet)
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => break 'outer,
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break 'outer,
                    _ => {}
                }
            }
        }

        // Zeichensatz leicht rotieren für dynamischeren Hintergrund
        charset_offset = (charset_offset + 1) % charset.len();

        // Regen aktualisieren
        for (i, col) in columns.iter_mut().enumerate() {
            if last_update[i].elapsed() < Duration::from_millis(col.speed) {
                continue;
            }
            last_update[i] = Instant::now();

            // Kopf eine Zeile nach unten
            col.head_y += 1;

            // Trail-Länge
            let trail_len = 10;
            for offset in 0..=trail_len {
                let y = col.head_y - offset;
                if y < 0 || y >= height_i16 {
                    continue;
                }
                let y_u16 = y as u16;

                // Helligkeit entlang des Trails (0 = Kopf, 1 = Ende)
                let t = offset as f32 / trail_len as f32;
                let color = colorset.gradient_color(1.0 - t);

                // Kopf heller/fetter
                let ch = random_char(&charset, charset_offset);
                let styled = if offset == 0 {
                    ch.with(color).bold()
                } else {
                    ch.with(color)
                };

                stdout
                    .queue(cursor::MoveTo(col.x, y_u16))?
                    .queue(PrintStyledContent(styled))?;
            }

            // Wenn Kopf unterhalb der Zielzeile ist, prüfen, ob wir ein Zeichen „einloggen“
            if col.head_y as u16 == target_y {
                let col_x = col.x;
                if col_x >= start_x && col_x < start_x + target_len {
                    let idx = (col_x - start_x) as usize;
                    if locked_chars[idx].is_none() {
                        // Dieses Zeichen wird jetzt Teil des Ziel-Strings
                        let target_ch = target.chars().nth(idx).unwrap_or(' ');
                        locked_chars[idx] = Some(target_ch);
                    }
                }
            }

            // Wenn Kopf unten raus ist, Spalte neu starten
            if col.head_y >= height_i16 + trail_len as i16 {
                col.head_y = rng.gen_range(-20..0);
                col.speed = rng.gen_range(40..120);
            }
        }

        // Ziel-String zeichnen (eingeloggte Zeichen hervorgehoben)
        for (i, ch) in target.chars().enumerate() {
            let x = start_x + i as u16;
            let y = target_y;

            let locked = locked_chars[i].is_some();
            let base_color = Color::White;
            let styled = if locked {
                // „größer“/deutlich: fett + sehr hell
                ch.with(base_color).bold()
            } else {
                ch.with(Color::DarkGrey)
            };

            stdout
                .queue(cursor::MoveTo(x, y))?
                .queue(PrintStyledContent(styled))?;
        }

        stdout.flush()?;
        thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    // Aufräumen
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
