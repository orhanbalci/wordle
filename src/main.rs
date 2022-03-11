use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Row, Table};
use figlet_rs::FIGfont;
use serde_json::json;
use std::fs::{self, File};
use std::time::Duration;
use std::{
    fmt::Display,
    io,
    io::{Read, Write},
};
use unicode_segmentation::UnicodeSegmentation;
mod api;
mod model;
mod tdk_api;

fn main() {
    let todays_puzzle = api::today().expect("can not retrieve todays puzzle");
    println!("{}. Gün Wordle Hoş Geldiniz", todays_puzzle.count);
    let _update_result = update_dictionary();
    let dictionary = read_dictionary();
    let mut gh = GuessHistory::new(
        String::from(&todays_puzzle.word),
        todays_puzzle.count,
        dictionary,
    );
    while !gh.is_over() {
        let user_input = print_prompt();
        let guess_result = gh.add_guess(user_input);
        match guess_result {
            Err(err) => print_error_message(err),
            Ok(_) => {
                println!("{}", gh);
                if gh.is_won() {
                    println!("Tebrikler kelimeyi doğru tahmin ettiniz");
                    println!(
                        "{}",
                        format!(
                            "{}: {}",
                            &todays_puzzle.word,
                            tdk_api::meaning(&todays_puzzle.word).unwrap_or_default()
                        )
                    );
                    gh.print_statistics();
                }
                if gh.is_failed() {
                    println!("Tahmin sayınız doldu. Maalesef kelimeyi bilemediniz.");
                }
            }
        }
    }
}

fn print_error_message(err: Errors) {
    match err {
        Errors::GuessLimitExceeded => println!("Tahmin sayınız doldu."),
        Errors::LengthMismatch => println!("Tahmin ile hedef kelime farklı uzunlukta."),
        Errors::InvalidGuessLength => println!("Kelime uzunluğu 5 harf olmalı."),
        Errors::NotInDictionary => println!("Kelime sözlükte bulunamadı."),
        _ => println!("Beklenmedik hata oluştu."),
    }
}

fn print_prompt() -> String {
    println!("Yeni tahmin >");
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("can not read user input");
    return buffer.trim().to_string();
}

fn update_dictionary() -> Result<bool, Errors> {
    if should_update().map_or(true, |f| f) {
        let dictionary = api::dictionary().map_err(|_e| Errors::CanNotGetDictionary)?;
        let file_content = json!({
            "words": dictionary.words
        })
        .to_string();
        if let Some(mut data_local) = dirs::data_local_dir() {
            data_local.push("wordle");
            if !data_local.exists() {
                fs::create_dir_all(&data_local).map_err(|_e| Errors::DataLocalNotCreated)?;
            }
            data_local.push("dictionary.json");
            let mut file =
                File::create(data_local).map_err(|_e| Errors::DictionaryFileNotCreated)?;
            file.write_all(file_content.as_bytes())
                .map_err(|_e| Errors::DictionaryFileNotWritten)?;
            return Ok(true);
        } else {
            return Err(Errors::DataLocalNotFound);
        }
    } else {
        return Ok(false);
    }
}

fn should_update() -> Result<bool, Errors> {
    let mut data_local = dirs::data_local_dir().ok_or(Errors::DataLocalNotFound)?;
    data_local.push("wordle");
    data_local.push("dictionary.json");
    let metadata = fs::metadata(data_local).map_err(|_e| Errors::CanNotReadMetaData)?;

    if let Ok(time) = metadata.accessed() {
        return time.elapsed().map_or(Ok(true), |elapsed| {
            Ok(elapsed > Duration::from_secs(60 * 60 * 24 * 7))
        });
    } else {
        return Ok(true);
    }
}

fn read_dictionary() -> Vec<String> {
    if let Some(mut data_local) = dirs::data_local_dir() {
        data_local.push("wordle");
        data_local.push("dictionary.json");
        return File::open(data_local)
            .map(|mut file| {
                let mut content = String::new();
                file.read_to_string(&mut content);
                content
            })
            .map(|content| {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    let words = v["words"].as_array();
                    if let Some(w) = words {
                        return w
                            .iter()
                            .map(|a| a.as_str().unwrap_or_default().into())
                            .collect::<Vec<String>>();
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default();
    }
    return Vec::new();
}

struct GuessHistory {
    target: String,
    guesses: Vec<String>,
    standart_font: FIGfont,
    wordle_day: u64,
    dictionary: Vec<String>,
}

#[derive(Debug)]
enum Errors {
    GuessLimitExceeded,
    LengthMismatch,
    InvalidGuessLength,
    DataLocalNotFound,
    DataLocalNotCreated,
    DictionaryFileNotCreated,
    DictionaryFileNotWritten,
    CanNotGetDictionary,
    CanNotReadMetaData,
    NotInDictionary,
}

impl GuessHistory {
    pub fn new(target: String, wordle_day: u64, dictionary: Vec<String>) -> Self {
        GuessHistory {
            target: target,
            guesses: Vec::new(),
            standart_font: FIGfont::standand().unwrap(),
            wordle_day: wordle_day,
            dictionary: dictionary,
        }
    }

    pub fn add_guess(&mut self, new_guess: String) -> Result<bool, Errors> {
        if new_guess.chars().count() != 5 {
            return Err(Errors::InvalidGuessLength);
        } else if self.guesses.len() > 5 {
            return Err(Errors::GuessLimitExceeded);
        } else if !self.dictionary.contains(&new_guess) {
            return Err(Errors::NotInDictionary);
        } else {
            self.guesses.push(new_guess);
            return Ok(true);
        }
    }

    pub fn count(&self) -> usize {
        self.guesses.len()
    }

    pub fn is_over(&self) -> bool {
        self.is_failed() || self.is_won()
    }

    pub fn is_won(&self) -> bool {
        if self.count() == 0 {
            return false;
        }
        return self
            .get_colors_index(self.count() - 1)
            .expect("can not get colors")
            .iter()
            .all(|c| *c == Color::Green);
    }

    pub fn print_statistics(&self) {
        println!("Wordle {} {}/6", self.wordle_day, self.guesses.len());
    }

    pub fn is_failed(&self) -> bool {
        self.guesses.len() == 6
    }

    pub fn get_colors_index(&self, index: usize) -> Result<Vec<Color>, Errors> {
        return GuessHistory::get_colors(&self.target, &self.guesses[index]);
    }

    fn get_colors(target: &str, needle: &str) -> Result<Vec<Color>, Errors> {
        if target.chars().count() != needle.chars().count()
            || target.chars().count() != 5
            || needle.chars().count() != 5
        {
            return Err(Errors::LengthMismatch);
        } else {
            let mut result = Vec::new();
            let needle_graphemes =
                UnicodeSegmentation::graphemes(needle, true).collect::<Vec<&str>>();
            let target_graphemes =
                UnicodeSegmentation::graphemes(target, true).collect::<Vec<&str>>();
            for (index, &c) in needle_graphemes.iter().enumerate() {
                let matches: Vec<_> = target_graphemes
                    .iter()
                    .enumerate()
                    .filter(|(_tindex, &tval)| tval == c)
                    .collect();
                if matches.len() > 0 {
                    if matches.iter().any(|(match_index, _)| *match_index == index) {
                        result.push(Color::Rgb {
                            r: 173,
                            g: 247,
                            b: 182,
                        });
                    } else {
                        result.push(Color::Rgb {
                            r: 255,
                            g: 238,
                            b: 147,
                        });
                    }
                } else {
                    result.push(Color::Rgb {
                        r: 255,
                        g: 192,
                        b: 159,
                    });
                }
            }
            return Ok(result);
        }
    }
}

impl Display for GuessHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut guess_table = Table::new();
        guess_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);
        for guess in &self.guesses {
            let colors =
                GuessHistory::get_colors(&self.target, guess).expect("can not calculate colors");
            let guess_cells = guess
                .chars()
                .zip(colors.iter())
                .map(|(c, color)| {
                    self.standart_font
                        .convert(&String::from(c))
                        .map_or(Cell::new(""), |s| Cell::new(&s.to_string()).bg(*color))
                })
                .collect::<Vec<Cell>>();
            guess_table.add_row(Row::from(guess_cells));
        }
        write!(f, "{guess_table}")
    }
}
