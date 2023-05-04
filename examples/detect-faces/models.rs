use std::path::PathBuf;

#[macro_export]
macro_rules! model_path {
    ($args:ident, $var:tt, $default:literal) => {
        crate::models::model_path($args.$var.as_ref(), $args.models_dir.as_ref(), $default)
    };
}

#[macro_export]
macro_rules! load_model {
    ($model:ident, $path:expr, $name:literal) => {
        $model::load({
            let file = std::fs::File::open($path).context(format!("Cannot find {} model file.", $name))?;
            std::io::BufReader::new(file)
        }).context(format!("Invalid {} model file.", $name))?
    }
}

#[macro_export]
macro_rules! detector {
    ($args:ident) => {
        load_model!(
            Detector,
            model_path!($args, face_finder, "face.detector.bin"),
            "face finder"
        )
    };
}

#[macro_export]
macro_rules! shaper {
    ($args:ident) => {
        load_model!(
            Shaper,
            model_path!($args, face_shaper, "face-5.shaper.bin"),
            "face shaper"
        )
    };
}

#[macro_export]
macro_rules! localizer {
    ($args:ident) => {
        load_model!(
            Localizer,
            model_path!($args, pupil_localizer, "pupil.localizer.bin"),
            "pupil localizer"
        )
    };
}

pub fn model_path<T: Into<PathBuf>>(
    input: Option<&PathBuf>,
    dir: Option<&PathBuf>,
    default: T,
) -> PathBuf {
    let path = input.map(|p| p.to_owned()).unwrap_or(default.into());
    if path.to_owned().is_absolute() {
        path
    } else {
        match dir {
            Some(p) => p.join(path),
            None => path,
        }
    }
}