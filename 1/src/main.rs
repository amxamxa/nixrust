use clap::Parser;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType};
use crossterm::ExecutableCommand;
use rand::Rng;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

/// Hex-Farbe in (r,g,b) konvertieren.
fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    (r, g, b)
}

/// Verfügbare Farbsets (Name -> Vec<RGB>)
fn colorsets() -> HashMap<String, Vec<(u8, u8, u8)>> {
    let mut map = HashMap::new();
    map.insert(
        "Determination".to_string(),
        vec!["#39c4b6", "#fee801", "#6300ff"]
            .into_iter()
            .map(hex_to_rgb)
            .collect(),
    );
    map.insert(
        "City".to_string(),
        vec!["#ff0677", "#0051ff", "#8900ff"]
            .into_iter()
            .map(hex_to_rgb)
            .collect(),
    );
    map.insert(
        "2077".to_string(),
        vec!["#c5003c", "#880425", "#f3e600", "#55ead4"]
            .into_iter()
            .map(hex_to_rgb)
            .collect(),
    );
    map.insert(
        "Thermography".to_string(),
        vec!["#ff004a", "#ffcc3d", "#ff5631", "#ad00ff"]
            .into_iter()
            .map(hex_to_rgb)
            .collect(),
    );
    map
}

/// Kommandozeilenargumente
#[derive(Parser)]
#[clap(author, version, about = "Matrix Digital Rain mit eingeblendetem Text")]
struct Args {
    /// Name des Farbsets (Determination, City, 2077, Thermography)
    #[arg(short, long)]
    colorset: Option<String>,

    /// Einzublendender Text (Standard: "Hallo Welt!")
    #[arg(short, long, default_value = "Hallo Welt!")]
    string: String,

    /// Liste aller verfügbaren Farbsets anzeigen
    #[arg(long, conflicts_with = "colorset")]
    list: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Farbsets initialisieren
    let sets = colorsets();

    // Falls nur Liste gewünscht
    if args.list {
        println!("Verfügbare Farbsets:");
        for name in sets.keys() {
            println!("  {}", name);
        }
        return Ok(());
    }

    // Gewähltes Farbset holen
    let colorset = args.colorset.as_deref().unwrap_or("Determination");
    let palette = sets
        .get(colorset)
        .ok_or_else(|| anyhow::anyhow!("Unbekanntes Farbset: {}", colorset))?;
    let max_age = palette.len(); // Anzahl Farbstufen

    // Terminal in Raw-Modus versetzen und Cursor verstecken
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(Hide)?;

    // Aktuelle Terminalgröße
    let (mut cols, mut rows) = size()?;
    let mut grid: Vec<Vec<Option<(char, usize)>>> = vec![vec![None; cols as usize]; rows as usize];

    // Zeichensatz für den Regen
    let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%&*()"
        .chars()
        .collect();

    // Wahrscheinlichkeit für neuen Tropfen pro Spalte und Frame
    let p_new = 0.3;

    // Hauptschleife
    loop {
        // Terminalgröße prüfen
        let (new_cols, new_rows) = size()?;
        if new_cols != cols || new_rows != rows {
            cols = new_cols;
            rows = new_rows;
            grid = vec![vec![None; cols as usize]; rows as usize];
        }

        // ---------- Regen aktualisieren ----------
        for x in 0..cols as usize {
            // Von unten nach oben durchgehen und Zeichen nach unten verschieben
            for y in (1..rows as usize).rev() {
                grid[y][x] = grid[y - 1][x].take().map(|(ch, age)| (ch, age + 1));
            }
            // Obere Zeile: neues Zeichen mit Wahrscheinlichkeit p_new
            if rand::thread_rng().gen::<f64>() < p_new {
                let ch = charset[rand::thread_rng().gen_range(0..charset.len())];
                grid[0][x] = Some((ch, 0));
            } else {
                grid[0][x] = None;
            }
        }

        // ---------- Bildschirm neu zeichnen ----------
        // Gesamten Bildschirm löschen und Cursor nach Hause
        stdout.execute(Clear(ClearType::All))?;

        // Jede Zeile des Regens ausgeben
        for y in 0..rows as usize {
            for x in 0..cols as usize {
                match grid[y][x] {
                    Some((ch, age)) => {
                        // Farbe basierend auf Alter (bei zu hohem Alter letzte Farbe)
                        let (r, g, b) = if age < max_age {
                            palette[age]
                        } else {
                            *palette.last().unwrap()
                        };
                        print!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, ch);
                    }
                    None => {
                        print!(" ");
                    }
                }
            }
            println!(); // Zeilenumbruch
        }

        // Text einblenden (zentriert, fett, weiß)
        let text = &args.string;
        let text_len = text.len();
        let text_row = rows as usize / 2;
        let text_col = (cols as usize - text_len) / 2;
        if text_row < rows as usize && text_col <= cols as usize {
            // Cursor positionieren (1‑basiert)
            print!("\x1b[{};{}H", text_row + 1, text_col + 1);
            print!("\x1b[1;38;2;255;255;255m{}\x1b[0m", text);
        }

        // Cursor wieder an den Anfang (damit er nicht blinkt)
        print!("\x1b[H");
        stdout.flush()?;

        // Auf Tastendruck prüfen
        if poll(Duration::from_millis(50))? {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => break,
                _ => {}
            }
        } else {
            // Keine Taste -> weiter
        }

        sleep(Duration::from_millis(50));
    }

    // Terminal wiederherstellen
    disable_raw_mode()?;
    stdout.execute(Show)?;
    stdout.execute(Clear(ClearType::All))?;
    print!("\x1b[H");
    stdout.flush()?;
    Ok(())
}
