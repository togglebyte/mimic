use std::collections::HashMap;
use std::fs::read_dir;
use std::path::PathBuf;

use kira::sound::static_sound::StaticSoundData;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};

use super::error::{Error, Result};

pub struct AudioShell {
    audio: Option<Audio>,
}

impl AudioShell {
    pub fn new() -> Self {
        Self { audio: None }
    }

    pub fn load(&mut self, path: PathBuf) -> Result<()> {
        self.audio = Some(Audio::load(path)?);
        Ok(())
    }

    pub fn play(&mut self, name: &str) {
        let Some(audio) = self.audio.as_mut() else { return };
        audio.play(name);
    }

    // pub fn set_volume(&mut self, vol: f32) {
    //     let Some(audio) = self.audio.as_mut() else { return };
    //     audio.set_volume(vol);
    // }
}

struct Audio {
    manager: AudioManager<DefaultBackend>,
    sounds: HashMap<String, StaticSoundData>,
    default: StaticSoundData,
}

impl Audio {
    pub fn load(root: PathBuf) -> Result<Self> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        let default = StaticSoundData::from_file(root.join("default.mp3")).map_err(|_| Error::NoDefaultSound)?;
        let sounds = load_sounds(root)?;

        let inst = Self {
            manager,
            sounds,
            default,
        };

        Ok(inst)
    }

    pub fn play(&mut self, name: &str) {
        let sound = match name {
            "\n" => self.get_sound("enter"),
            " " => self.get_sound("space"),
            ";" | "<" | ">" => self.get_sound("semicolon"),
            name => self.get_sound(name),
        };
        _ = self.manager.play(sound);
    }

    fn get_sound(&self, name: &str) -> StaticSoundData {
        self.sounds.get(name).unwrap_or(&self.default).clone()
    }

    // pub fn set_volume(&mut self, vol: f32) {
    //     self.default.volume(vol);
    //     for sound in self.sounds.values() {
    //         sound.volume(vol);
    //     }
    // }
}

fn load_sounds(path: PathBuf) -> Result<HashMap<String, StaticSoundData>> {
    let mut entries = read_dir(&path).map_err(|_| Error::FilePath(path))?;
    let mut hm = HashMap::new();

    while let Some(Ok(entry)) = entries.next() {
        let path = entry.path();
        let Some(name) = path.file_stem() else { continue };
        let Some(name) = name.to_str() else { continue };
        let name = name.to_string();
        let Ok(sound) = StaticSoundData::from_file(path) else { continue };
        hm.insert(name, sound);
    }

    Ok(hm)
}
