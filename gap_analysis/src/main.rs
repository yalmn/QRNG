use serde_json::Value;
use std::env;
use std::fs;

mod randomness;
use randomness::RandomnessAnalyzer;

/// JSON einlesen und Feld "qrn" extrahieren.
/// Erwartet z.B.: {"length":3,"qrn":"[11011111 10000100 00011001]"}
fn parse_file_to_qrn_string(path: &str) -> String {
    let raw_data = fs::read_to_string(path).expect("Failed to read file.\n");
    let parsed: Value = serde_json::from_str(&raw_data).expect("Failed to parse JSON.\n");

    parsed
        .get("qrn")
        .and_then(|v| v.as_str())
        .expect("Missing field 'qrn'.\n")
        .to_owned()
}

/// Harmonisiert einen qrn-String aus JSON:
/// - entfernt '[' , ']' und Whitespace
/// Ergebnis enthält nur 0/1 als zusammenhängenden Bitstream
fn harmonize_qrn_from_json(qrn: &str) -> String {
    qrn.chars()
        .filter(|c| *c == '0' || *c == '1')
        .collect()
}

/// Liest eine bereits harmonisierte Datei (oder eine mit Whitespace),
/// die direkt Bits enthält wie:
/// 1001010100101010...
///
/// Erlaubt nur '0','1' und Whitespace. Alles andere => Fehler.
fn read_raw_bitstream_file(path: &str) -> String {
    let raw = fs::read_to_string(path).expect("Failed to read raw bitstream file.\n");

    let mut out = String::with_capacity(raw.len());
    for (idx, c) in raw.chars().enumerate() {
        match c {
            '0' | '1' => out.push(c),
            ' ' | '\n' | '\r' | '\t' => {}
            _ => panic!("Invalid character '{}' at position {} in raw bitstream.", c, idx),
        }
    }

    if out.is_empty() {
        panic!("Raw bitstream file contains no 0/1.");
    }

    out
}

/// Heuristik: Datei ist JSON, wenn erstes nicht-Whitespace Zeichen '{' ist
fn is_json_file_content(s: &str) -> bool {
    s.chars().find(|c| !c.is_whitespace()) == Some('{')
}

/// Zählt (ones, zeros) im Bitstring (nur '0'/'1' werden gezählt)
fn quantity_oz(bitstream: &str) -> [i32; 2] {
    let mut zeros: i32 = 0;
    let mut ones: i32 = 0;

    for c in bitstream.chars() {
        match c {
            '0' => zeros += 1,
            '1' => ones += 1,
            _ => {}
        }
    }

    // res[0] = ones, res[1] = zeros (wie du es bisher genutzt hast)
    [ones, zeros]
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.json|file.bin|file.txt>\n", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];

    // Datei einmal lesen, um zu entscheiden ob JSON oder raw-bitstream
    let file_content = fs::read_to_string(path).expect("Failed to read file.\n");

    let bitstream: String = if is_json_file_content(&file_content) {
        // JSON-Modus: qrn extrahieren + harmonisieren (nur 0/1 behalten)
        let qrn = {
            let parsed: Value =
                serde_json::from_str(&file_content).expect("Failed to parse JSON.\n");
            parsed
                .get("qrn")
                .and_then(|v| v.as_str())
                .expect("Missing field 'qrn'.\n")
                .to_owned()
        };
        harmonize_qrn_from_json(&qrn)
    } else {
        // Raw-Modus: Datei enthält schon den Bitstream (evtl. mit whitespace)
        // (wir lesen sie nochmal strikt, damit invalid chars sauber knallen)
        read_raw_bitstream_file(path)
    };

    // Ausgabe wie bisher (optional)
    //println!("{}", bitstream);

    let res = quantity_oz(&bitstream);
    // Achtung: res[0]=ones, res[1]=zeros (du hattest bisher die Labels vertauscht)
    //println!("0: {}, 1: {}\n", res[1], res[0]);

    // Metriken berechnen
    let analyzer = RandomnessAnalyzer::new(&bitstream)
        .expect("Invalid bitstream (must contain only 0/1)");

    println!("len = {}", bitstream.len());
    println!("p_e = {:.6}", analyzer.bop());

    let gaps = analyzer.gaps();
    println!("gaps_count = {}", gaps.len());
    //println!("gaps = {:?}", gaps);

    //println!("v(k) = {:?}", analyzer.gap_density());
    //println!("u(k) = {:?}", analyzer.gap_distribution());

    let (p, v0, diff) = analyzer.iid_du_check();
    println!(
        "IID-DU: p_e={:.6}, v(0)={:.6}, |diff|={:.6}",
        p, v0, diff
    );

    let bursts = analyzer.bursts(3);
    //println!("bursts(a=3) = {:?}", bursts);

    println!("B = {:?}", analyzer.burstiness_level());
}
