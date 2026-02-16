use clap::{Parser, ValueEnum};
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, PrintStyledContent, Stylize},
    terminal::{self, ClearType},
};
use rand::Rng;
use std::cmp::min;
use std::io::{Write, stdout};
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
            ColorSetName::C2077 => Self::from_hex(&["#c5003c", "#880425", "#f3e600", "#55ead4"]),
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
#[command(
    name = "matrix",
    version,
    about = "Retro-futuristischer Matrix-Digital-Rain in Rust"
)]
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

    /// Hintergrund-Verschiebungsgeschwindigkeit (0-10)
    #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u8).range(0..=10))]
    scroll_speed: u8,
}

#[derive(Clone)]
struct Column {
    x: u16,
    head_y: i16,
    speed: u64,
    phase: usize,
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
    let scroll_speed = args.scroll_speed;

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    let (width, height) = terminal::size()?;
    let height_i16 = height as i16;

    // Ziel-String in integrierter 3x5-Schrift
    let figlet_lines = render_3x5(&target);
    let mut target_lines: Vec<Vec<char>> =
        figlet_lines.iter().map(|l| l.chars().collect()).collect();
    let target_height = target_lines.len().max(1) as u16;
    let target_width = target_lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
    for line in target_lines.iter_mut() {
        if line.len() < target_width as usize {
            line.extend(std::iter::repeat_n(' ', target_width as usize - line.len()));
        }
    }

    // Ziel-Block zentrieren
    let start_x = if target_width < width {
        (width - target_width) / 2
    } else {
        0
    };
    let max_y = height.saturating_sub(1);
    let target_y = (height.saturating_sub(target_height) / 2).min(max_y);
    let border_y0 = target_y.saturating_sub(1);
    let border_y1 = (target_y + target_height).min(max_y);
    let max_x = width.saturating_sub(1);
    let border_x0 = start_x.saturating_sub(1);
    let border_x1 = (start_x + target_width).min(max_x);

    // Für jedes Zeichen im Ziel-String merken wir, ob es schon „eingeloggt“ ist
    let mut locked_chars: Vec<Vec<Option<char>>> =
        vec![vec![None; target_width as usize]; target_height as usize];

    // Zeichensatz für Regen
    let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    let mut frame: usize = 0;
    let mut bg_shift: u16 = 0;
    let mut bg_tick: u16 = 0;
    let scroll_interval: u16 = if scroll_speed == 0 {
        u16::MAX
    } else {
        11 - scroll_speed as u16
    };

    // Spalten initialisieren
    let mut rng = rand::thread_rng();
    let mut columns: Vec<Column> = (0..width)
        .map(|x| Column {
            x,
            head_y: rng.gen_range(-20..0),
            speed: rng.gen_range(40..120), // ms pro Schritt
            phase: rng.gen_range(0..charset.len()),
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
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => break 'outer,
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break 'outer,
                    _ => {}
                }
            }
        }

        // Frame-Tick für durchlaufende Zeichenrotation
        frame = frame.wrapping_add(1);
        bg_tick = bg_tick.wrapping_add(1);
        if bg_tick.is_multiple_of(scroll_interval) {
            bg_shift = (bg_shift + 1) % width.max(1);
        }

        // Regen aktualisieren
        for (i, col) in columns.iter_mut().enumerate() {
            if last_update[i].elapsed() < Duration::from_millis(col.speed) {
                continue;
            }
            last_update[i] = Instant::now();
            col.phase = (col.phase + 1) % charset.len();

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
                let draw_x = (col.x + bg_shift) % width.max(1);
                let in_target_area = draw_x >= border_x0
                    && draw_x <= border_x1
                    && y_u16 >= border_y0
                    && y_u16 <= border_y1;
                if in_target_area {
                    continue;
                }

                // Helligkeit entlang des Trails (0 = Kopf, 1 = Ende)
                let t = offset as f32 / trail_len as f32;
                let color = colorset.gradient_color(1.0 - t);

                // Kopf heller/fetter
                let ch =
                    charset[(frame + col.phase + col.x as usize + offset as usize) % charset.len()];
                let styled = if offset == 0 {
                    ch.with(color).bold()
                } else {
                    ch.with(color)
                };

                stdout
                    .queue(cursor::MoveTo(draw_x, y_u16))?
                    .queue(PrintStyledContent(styled))?;
            }

            // Wenn Kopf unterhalb der Zielzeile ist, prüfen, ob wir ein Zeichen „einloggen“
            let col_x = (col.x + bg_shift) % width.max(1);
            if col_x >= start_x && col_x < start_x + target_width {
                let row = col.head_y as i32 - target_y as i32;
                if row >= 0 && (row as u16) < target_height {
                    let row_idx = row as usize;
                    let col_idx = (col_x - start_x) as usize;
                    if locked_chars[row_idx][col_idx].is_none() {
                        let target_ch = target_lines
                            .get(row_idx)
                            .and_then(|line| line.get(col_idx))
                            .copied()
                            .unwrap_or(' ');
                        if target_ch != ' ' {
                            locked_chars[row_idx][col_idx] = Some(target_ch);
                        }
                    }
                }
            }

            // Wenn Kopf unten raus ist, Spalte neu starten
            if col.head_y >= height_i16 + trail_len {
                col.head_y = rng.gen_range(-20..0);
                col.speed = rng.gen_range(40..120);
                col.phase = rng.gen_range(0..charset.len());
            }
        }

        // Rahmen zeichnen
        if width > 0 && height > 0 {
            let border_style = '+'.with(Color::DarkGrey);
            let horiz_style = '-'.with(Color::DarkGrey);
            let vert_style = '|'.with(Color::DarkGrey);

            if border_x0 <= border_x1 {
                for x in border_x0..=border_x1 {
                    let ch = if (x == border_x0 || x == border_x1)
                        && (border_y0 == border_y1
                            || border_y0 == target_y
                            || border_y1 == target_y)
                    {
                        border_style
                    } else {
                        horiz_style
                    };
                    stdout
                        .queue(cursor::MoveTo(x, border_y0))?
                        .queue(PrintStyledContent(ch))?;
                    if border_y1 != border_y0 {
                        stdout
                            .queue(cursor::MoveTo(x, border_y1))?
                            .queue(PrintStyledContent(ch))?;
                    }
                }
            }
            if border_y0 < border_y1.saturating_sub(1) && border_x0 <= border_x1 {
                for y in (border_y0 + 1)..=border_y1.saturating_sub(1) {
                    stdout
                        .queue(cursor::MoveTo(border_x0, y))?
                        .queue(PrintStyledContent(vert_style))?;
                    if border_x1 != border_x0 {
                        stdout
                            .queue(cursor::MoveTo(border_x1, y))?
                            .queue(PrintStyledContent(vert_style))?;
                    }
                }
            }
        }

        // Ziel-String zeichnen (eingeloggte Zeichen hervorgehoben)
        for (row, line) in target_lines.iter().enumerate() {
            let y = target_y + row as u16;
            for (col, ch) in line.iter().enumerate() {
                if *ch == ' ' {
                    continue;
                }
                let x = start_x + col as u16;
                let locked = locked_chars[row][col].is_some();
                let base_color = Color::White;
                let styled = if locked {
                    ch.with(base_color).bold()
                } else {
                    ch.with(Color::DarkGrey)
                };
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(PrintStyledContent(styled))?;
            }
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

fn render_3x5(input: &str) -> Vec<String> {
    let mut rows = vec![
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];

    for ch in input.chars() {
        let glyph = glyph_3x5(ch);
        for (row, pattern) in rows.iter_mut().zip(glyph.iter()) {
            row.push_str(pattern);
            row.push(' '); // Abstand zwischen Zeichen
        }
    }

    while rows.last().is_some_and(|r| r.is_empty()) {
        rows.pop();
    }

    rows
}

fn glyph_3x5(ch: char) -> [&'static str; 5] {
    match ch.to_ascii_uppercase() {
        'A' => ["###", "# #", "###", "# #", "# #"],
        'B' => ["## ", "# #", "## ", "# #", "## "],
        'C' => ["###", "#  ", "#  ", "#  ", "###"],
        'D' => ["## ", "# #", "# #", "# #", "## "],
        'E' => ["###", "#  ", "###", "#  ", "###"],
        'F' => ["###", "#  ", "###", "#  ", "#  "],
        'G' => ["###", "#  ", "# #", "# #", "###"],
        'H' => ["# #", "# #", "###", "# #", "# #"],
        'I' => ["###", " # ", " # ", " # ", "###"],
        'J' => ["###", "  #", "  #", "# #", "###"],
        'K' => ["# #", "## ", "#  ", "## ", "# #"],
        'L' => ["#  ", "#  ", "#  ", "#  ", "###"],
        'M' => ["# #", "###", "###", "# #", "# #"],
        'N' => ["# #", "###", "###", "###", "# #"],
        'O' => ["###", "# #", "# #", "# #", "###"],
        'P' => ["###", "# #", "###", "#  ", "#  "],
        'Q' => ["###", "# #", "# #", "###", "  #"],
        'R' => ["###", "# #", "###", "## ", "# #"],
        'S' => ["###", "#  ", "###", "  #", "###"],
        'T' => ["###", " # ", " # ", " # ", " # "],
        'U' => ["# #", "# #", "# #", "# #", "###"],
        'V' => ["# #", "# #", "# #", "# #", " # "],
        'W' => ["# #", "# #", "###", "###", "# #"],
        'X' => ["# #", "# #", " # ", "# #", "# #"],
        'Y' => ["# #", "# #", " # ", " # ", " # "],
        'Z' => ["###", "  #", " # ", "#  ", "###"],
        '0' => ["###", "# #", "# #", "# #", "###"],
        '1' => [" # ", "## ", " # ", " # ", "###"],
        '2' => ["###", "  #", "###", "#  ", "###"],
        '3' => ["###", "  #", "###", "  #", "###"],
        '4' => ["# #", "# #", "###", "  #", "  #"],
        '5' => ["###", "#  ", "###", "  #", "###"],
        '6' => ["###", "#  ", "###", "# #", "###"],
        '7' => ["###", "  #", " # ", " # ", " # "],
        '8' => ["###", "# #", "###", "# #", "###"],
        '9' => ["###", "# #", "###", "  #", "###"],
        '!' => [" # ", " # ", " # ", "   ", " # "],
        '?' => ["###", "  #", " # ", "   ", " # "],
        '.' => ["   ", "   ", "   ", "   ", " # "],
        ',' => ["   ", "   ", "   ", " # ", "#  "],
        '-' => ["   ", "   ", "###", "   ", "   "],
        '_' => ["   ", "   ", "   ", "   ", "###"],
        ':' => ["   ", " # ", "   ", " # ", "   "],
        '/' => ["  #", "  #", " # ", "#  ", "#  "],
        ' ' => ["   ", "   ", "   ", "   ", "   "],
        _ => ["###", " # ", "###", " # ", "###"],
    }
}
