######  Kompilieren:
  (Rust und Cargo müssen installiert sein):
```sh
    
    cargo new matrix_rain
    cd matrix_rain
    # Cargo.toml mit den Abhängigkeiten ergänzen (siehe unten)
    # Obigen Code in src/main.rs einfügen
    cargo build --release
    
```
###### Ausführen:
```sh
    
    ./target/release/matrix_rain
    ./target/release/matrix_rain -c City -s "Willkommen"
    ./target/release/matrix_rain --colorset 2077 --string "Cyberpunk"
    ./target/release/matrix_rain --list   # Verfügbare Farbsets anzeigen
    
. **Beenden**: `q` oder `Ctrl+C`.
``` 
   

## Cargo.toml

```toml
[package]
name = "matrix_rain"
version = "0.1.0"
edition = "2021"
[dependencies]
clap = { version = "4.0", features = ["derive"] }
crossterm = "0.27"
rand = "0.8"
anyhow = "1.0"

```
## Erläuterungen

- **Regenlogik**: Ein zweidimensionales `grid` speichert für jede Zelle entweder `None` (leer) oder ein Tupel `(Zeichen, Alter)`. Jeder Frame werden alle Zeilen nach unten verschoben (Alter +1) und oben mit einer Wahrscheinlichkeit von 30 % ein neues Zeichen erzeugt.
    
- **Farbgebung**: Die Helligkeit wird durch das Alter bestimmt. Alter 0 → erste Farbe der Palette (hellste), Alter 1 → zweite Farbe, … Ist das Alter größer als die Palettenlänge, wird die letzte (dunkelste) Farbe verwendet. Die Farben werden als 24‑Bit‑ANSI‑Codes ausgegeben.
    
- **Texteinblendung**: Nach dem Zeichnen des Regens wird der übergebene String in der Bildschirmmitte positioniert und in **fettem Weiß** ausgegeben – er hebt sich deutlich vom grünen Regen ab.
    
- **Terminal‑Handling**: `crossterm` setzt den Terminal in den Raw‑Modus, versteckt den Cursor und reagiert auf Größenänderungen (`size()` wird in jedem Frame neu abgefragt). Tastendrücke werden nicht blockierend mit `poll()` abgefragt.
    
- **Farbsets**: Die vier vorgegebenen Paletten sind als `HashMap` hinterlegt und können über `--colorset` ausgewählt werden. Mit `--list` werden alle verfügbaren Namen angezeigt.
    

Das Ergebnis ist ein authentischer, hypnotischer Matrix‑Regen, der durch den eingeblendeten Text individuell angepasst werden kann und sich jeder Terminalgröße anpasst.
