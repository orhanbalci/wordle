use crate::model::*;
use anyhow::Result;

pub fn dictionary() -> Result<Dictionary> {
    reqwest::blocking::get("https://wordle-backend.orhanbalci.workers.dev/words/tr")?
        .json::<Dictionary>()
        .map_err(|err| err.into())
}

pub fn today() -> Result<Daily> {
    reqwest::blocking::get("https://wordle-backend.orhanbalci.workers.dev/word/today/tr")?
        .json::<Daily>()
        .map_err(|err| err.into())
}

pub fn _previous() -> Result<Previous> {
    reqwest::blocking::get("https://wordle-backend.orhanbalci.workers.dev/word/previous/tr")?
        .json::<Previous>()
        .map_err(|err| err.into())
}
