//! Exporting a list from its originals (F024): the presets (S01), and the job that renders each
//! photo at a preset, or copies the originals, into a folder, numbered by date taken (S02).

use crate::db::Item;
use crate::manifest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub label: &'static str,
    /// What it is for, in the dialog.
    pub what: &'static str,
    /// The long side in pixels, never enlarged; `None`: the original's size.
    pub long: Option<usize>,
    pub quality: f64,
    pub dpi: Option<u32>,
    /// The originals as they are, with their sidecars, instead of a JPEG.
    pub originals: bool,
}

pub const PRESETS: &[Preset] = &[
    Preset { id: "delen", label: "Delen", what: "mail, berichten, een scherm", long: Some(2048), quality: 0.85, dpi: None, originals: false },
    Preset { id: "10x15", label: "Afdruk 10×15", what: "10 × 15 cm aan 300 dpi", long: Some(1800), quality: 0.92, dpi: Some(300), originals: false },
    Preset { id: "13x18", label: "Afdruk 13×18", what: "13 × 18 cm aan 300 dpi", long: Some(2126), quality: 0.92, dpi: Some(300), originals: false },
    Preset { id: "20x30", label: "Afdruk 20×30", what: "20 × 30 cm aan 300 dpi", long: Some(3543), quality: 0.92, dpi: Some(300), originals: false },
    Preset { id: "fotoboek-a4", label: "Fotoboek A4", what: "een pagina van een A4-fotoboek aan 300 dpi", long: Some(3508), quality: 0.92, dpi: Some(300), originals: false },
    Preset { id: "kalender-a3", label: "Kalender A3", what: "een A3-kalenderblad aan 300 dpi", long: Some(4961), quality: 0.92, dpi: Some(300), originals: false },
    Preset { id: "vol", label: "Volle resolutie", what: "JPEG op de grootte van het origineel; RAW door macOS ontwikkeld", long: None, quality: 0.95, dpi: None, originals: false },
    Preset { id: "originelen", label: "Originelen", what: "de bestanden zelf, ook RAW en video, met hun sidecars", long: None, quality: 0.0, dpi: None, originals: true },
];

pub fn preset(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}

/// An own setting (F024/S03, Peter 2026-11-05): a long side, a quality and a DPI chosen by hand.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Custom {
    pub long: usize,
    /// 1–100.
    pub quality: u8,
    pub dpi: Option<u32>,
}

impl Custom {
    /// A preset made of it, checked: 100–20 000 px, quality 1–100, DPI 72–1 200.
    pub fn preset(&self) -> Result<Preset, String> {
        if !(100..=20_000).contains(&self.long) {
            return Err("de lange zijde: tussen 100 en 20 000 pixels".into());
        }
        if !(1..=100).contains(&self.quality) {
            return Err("de kwaliteit: tussen 1 en 100".into());
        }
        if self.dpi.is_some_and(|d| !(72..=1200).contains(&d)) {
            return Err("de dpi: tussen 72 en 1 200".into());
        }
        Ok(Preset { id: "eigen", label: "Eigen instelling", what: "", long: Some(self.long), quality: f64::from(self.quality) / 100.0, dpi: self.dpi, originals: false })
    }
}

/// One file of an export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Part {
    /// The original, relative to `Library`.
    pub src: String,
    /// The name in the export folder.
    pub name: String,
    /// Rendered as a JPEG, or copied as it is (a video, the `originelen` preset, a sidecar).
    pub render: bool,
    pub taken: Option<String>,
    pub camera: Option<String>,
    #[serde(default)]
    pub done: bool,
    /// Lightroom's version to render from (F025/S05): `<src>.<variant>.full.jpg` in `Previews`
    /// on the NAS; empty: the original.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub variant: String,
}

/// The export's files (S02): the photos and videos by date taken, then name, numbered; Live
/// Photos' clips left out; with `originelen` the originals with their sidecars under the number.
/// `variant` gives each photo's chosen version (F025/S05); only a rendered photo uses it.
pub fn parts(items: &[Item], preset: &Preset, sidecars: &dyn Fn(&str) -> Vec<String>, variant: &dyn Fn(&Item) -> String) -> Vec<Part> {
    let mut media: Vec<&Item> = items.iter().filter(|i| (i.kind == "photo" || i.kind == "video") && i.live.is_none()).collect();
    media.sort_by(|a, b| a.taken.cmp(&b.taken).then_with(|| file_name(&a.path).cmp(file_name(&b.path))));
    let width = media.len().to_string().len().max(3);
    let mut out = Vec::new();
    for (n, i) in media.iter().enumerate() {
        let number = format!("{:0width$}", n + 1);
        let render = !preset.originals && i.kind == "photo";
        let name = if render {
            format!("{number}-{}.jpg", stem(&i.path))
        } else {
            format!("{number}-{}", file_name(&i.path))
        };
        let variant = if render { variant(i) } else { String::new() };
        out.push(Part { src: i.path.clone(), name, render, taken: i.taken.clone(), camera: i.camera.clone(), done: false, variant });
        if preset.originals {
            for sc in sidecars(&i.path) {
                out.push(Part { name: format!("{number}-{}", file_name(&sc)), src: sc, render: false, taken: None, camera: None, done: false, variant: String::new() });
            }
        }
    }
    out
}

fn file_name(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

fn stem(p: &str) -> &str {
    let f = file_name(p);
    f.rsplit_once('.').map_or(f, |(s, _)| s)
}

/// `Exports/<year>/<year>-<month>-<list slug>[-<preset>]` (data contract 6), for the moment `now`.
pub fn folder_name(list_slug: &str, preset: &Preset, now: &chrono::DateTime<chrono::Local>) -> String {
    let suffix = match (preset.id, preset.long) {
        ("vol", _) => String::new(),
        ("eigen", Some(long)) => format!("-{long}px"),
        (id, _) => format!("-{id}"),
    };
    format!("{}/{}-{}{suffix}", now.format("%Y"), now.format("%Y-%m"), list_slug)
}

/// An export under way, kept in the app's state so it continues after an interruption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub list: String,
    pub list_name: String,
    pub preset: String,
    /// The folder the files go to, absolute.
    pub dest: String,
    pub by: String,
    /// The own setting, when `preset` is `eigen`.
    #[serde(default)]
    pub custom: Option<Custom>,
    pub parts: Vec<Part>,
    pub problems: Vec<String>,
}

pub const EXPORT_DIR: &str = "export";

impl Job {
    fn path(config: &Path, id: &str) -> PathBuf {
        config.join(EXPORT_DIR).join(format!("{id}.json"))
    }

    pub fn save(&self, config: &Path) -> Result<(), String> {
        let p = Job::path(config, &self.id);
        fs::create_dir_all(p.parent().unwrap_or(config)).map_err(|e| e.to_string())?;
        manifest::write(&p, &serde_json::to_value(self).map_err(|e| e.to_string())?)
    }

    pub fn pending(config: &Path) -> Option<Job> {
        let rd = fs::read_dir(config.join(EXPORT_DIR)).ok()?;
        let mut jobs: Vec<Job> = rd.flatten().filter_map(|e| serde_json::from_slice(&fs::read(e.path()).ok()?).ok()).collect();
        jobs.sort_by(|a, b| a.id.cmp(&b.id));
        jobs.into_iter().next()
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
}

/// Copies through a temporary name, content and time only.
fn copy(from: &Path, to: &Path) -> Result<(), String> {
    let tmp = to.with_extension(format!("{}.tmp", to.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default()));
    let r = (|| -> std::io::Result<()> {
        let t = fs::metadata(from)?.modified()?;
        let mut out = fs::File::create(&tmp)?;
        std::io::copy(&mut fs::File::open(from)?, &mut out)?;
        out.sync_all()?;
        out.set_modified(t)?;
        fs::rename(&tmp, to)
    })();
    r.map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("{}: {e}", from.display())
    })
}

/// Runs the export (S02): each part from the working copy, or from `nas` when it isn't here,
/// rendered or copied into `dest`; a file there already is kept; the journal saved after each;
/// `export.json` at the end (data contract 6).
/// `previews_nas`: the NAS's `Previews`, where Lightroom's full versions are (F025/S05).
pub fn run(job: &mut Job, here: &Path, nas: Option<&Path>, previews_nas: Option<&Path>, config: &Path, render: &dyn Fn(&Path, &Part, &Path) -> Result<(), String>, progress: &mut dyn FnMut(&Progress)) -> Result<(), String> {
    let dest = PathBuf::from(&job.dest);
    fs::create_dir_all(&dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    job.save(config)?;
    let mut p = Progress { done: job.parts.iter().filter(|x| x.done).count(), total: job.parts.len() };
    progress(&p);
    for i in 0..job.parts.len() {
        if job.parts[i].done {
            continue;
        }
        let part = job.parts[i].clone();
        let out = dest.join(&part.name);
        let local = here.join(&part.src);
        let edited = (!part.variant.is_empty())
            .then(|| previews_nas.map(|n| n.join(format!("{}.{}.full.jpg", part.src, part.variant))))
            .flatten()
            .filter(|p| p.is_file());
        if !part.variant.is_empty() && edited.is_none() && !out.exists() {
            job.problems.push(format!("{}: de bewerkte versie staat niet op de NAS, het origineel genomen", part.src));
            job.parts[i].variant.clear();
        }
        let src = if edited.is_some() {
            edited
        } else if local.is_file() {
            Some(local)
        } else {
            nas.map(|n| n.join(&part.src)).filter(|p| p.is_file())
        };
        let result = match (&src, out.exists()) {
            (_, true) => Ok(()),
            (None, _) => Err(format!("{}: niet op deze Mac en niet op de NAS", part.src)),
            (Some(s), _) if part.render => render(s, &part, &out),
            (Some(s), _) => copy(s, &out),
        };
        if let Err(e) = result {
            job.problems.push(e);
        }
        job.parts[i].done = true;
        p.done += 1;
        job.save(config)?;
        progress(&p);
    }
    let info = serde_json::json!({
        "contract_version": "1.0",
        "name": job.list_name,
        "from_lists": [job.list],
        "preset": job.preset,
        "exported_at": manifest::local_iso(chrono::Local::now()),
        "by": job.by,
    });
    let mut info = info;
    let edited: Vec<&str> = job.parts.iter().filter(|p| !p.variant.is_empty()).map(|p| p.name.as_str()).collect();
    if !edited.is_empty() {
        info["lightroom_edits"] = serde_json::json!(edited);
    }
    manifest::write(&dest.join("export.json"), &info)?;
    let _ = fs::remove_file(Job::path(config, &job.id));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(path: &str, kind: &str, taken: &str, live: Option<&str>) -> Item {
        Item {
            path: path.into(), folder: "2026/07".into(), kind: kind.into(), taken: Some(taken.into()), taken_from: None, source: None,
            size: 1, hash: "sha256:x".into(), local: true, preview: true, live: live.map(str::to_string), camera: Some("Canon EOS R".into()),
            variants: Vec::new(),
        }
    }

    fn items() -> Vec<Item> {
        vec![
            item("2026/07/Cam/b.CR3", "photo", "2026-07-30T10:00:00+02:00", None),
            item("2026/07/Cam/a.CR3", "photo", "2026-07-30T10:00:00+02:00", None),
            item("2026/07/Phone/IMG_1.MOV", "video", "2026-07-29T09:00:00+02:00", None),
            item("2026/07/Phone/IMG_2.MOV", "video", "2026-07-31T09:00:00+02:00", Some("2026/07/Phone/IMG_2.HEIC")),
            item("2026/07/Cam/a.xmp", "sidecar", "2026-07-30T10:00:00+02:00", None),
        ]
    }

    #[test]
    fn numbered_by_date_then_name_without_live_clips() {
        let none = |_: &str| vec![];
        let plain = |_: &Item| String::new();
        let p = parts(&items(), preset("fotoboek-a4").unwrap(), &none, &plain);
        let names: Vec<(&str, bool)> = p.iter().map(|x| (x.name.as_str(), x.render)).collect();
        assert_eq!(names, [("001-IMG_1.MOV", false), ("002-a.jpg", true), ("003-b.jpg", true)]);
        let with_sidecars = |p: &str| if p.ends_with("a.CR3") { vec!["2026/07/Cam/a.xmp".to_string()] } else { vec![] };
        let o = parts(&items(), preset("originelen").unwrap(), &with_sidecars, &|_| "lr".into());
        assert_eq!(o.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["001-IMG_1.MOV", "002-a.CR3", "002-a.xmp", "003-b.CR3"]);
        assert!(o.iter().all(|x| !x.render && x.variant.is_empty()), "originals never take a version");
    }

    #[test]
    fn the_folder_and_the_presets() {
        let now = chrono::Local::now();
        let y = now.format("%Y").to_string();
        assert_eq!(folder_name("aosta", preset("fotoboek-a4").unwrap(), &now), format!("{y}/{}-aosta-fotoboek-a4", now.format("%Y-%m")));
        assert_eq!(folder_name("aosta", preset("vol").unwrap(), &now), format!("{y}/{}-aosta", now.format("%Y-%m")));
        assert_eq!(preset("10x15").unwrap().long, Some(1800));
        assert!(preset("nope").is_none());
        // an own setting, checked, with its size in the folder's name
        let own = Custom { long: 2400, quality: 90, dpi: Some(240) }.preset().unwrap();
        assert_eq!((own.long, own.quality, own.dpi), (Some(2400), 0.9, Some(240)));
        assert_eq!(folder_name("aosta", &own, &now), format!("{y}/{}-aosta-2400px", now.format("%Y-%m")));
        assert!(Custom { long: 50, quality: 90, dpi: None }.preset().is_err());
        assert!(Custom { long: 2400, quality: 0, dpi: None }.preset().is_err());
        assert!(Custom { long: 2400, quality: 90, dpi: Some(5000) }.preset().is_err());
    }

    #[test]
    fn run_resumed_from_here_or_the_nas_with_export_json() {
        let d = tempfile::tempdir().unwrap();
        let (here, nas, cfg, dest) = (d.path().join("here"), d.path().join("nas"), d.path().join("cfg"), d.path().join("Exports/2026/2026-11-aosta"));
        for (root, rel) in [(&here, "2026/07/Cam/a.CR3"), (&nas, "2026/07/Cam/b.CR3"), (&nas, "2026/07/Phone/IMG_1.MOV")] {
            fs::create_dir_all(root.join(rel).parent().unwrap()).unwrap();
            fs::write(root.join(rel), rel).unwrap();
        }
        let none = |_: &str| vec![];
        let mut job = Job {
            id: "t".into(), list: "2026/aosta".into(), list_name: "Aosta".into(), preset: "fotoboek-a4".into(), dest: dest.display().to_string(),
            by: "peter".into(), custom: None, problems: vec![],
            // a.CR3 edited in Lightroom, its full version on the NAS; b.CR3's chosen version missing
            parts: parts(&items(), preset("fotoboek-a4").unwrap(), &none, &|i: &Item| if i.path.ends_with(".CR3") { "lr".into() } else { String::new() }),
        };
        // as if an earlier run made the first file
        fs::create_dir_all(&dest).unwrap();
        fs::write(dest.join("001-IMG_1.MOV"), b"made before").unwrap();
        let fake = |src: &Path, _: &Part, out: &Path| fs::write(out, fs::read(src).unwrap()).map_err(|e| e.to_string());
        let mut last = Progress::default();
        let previews = d.path().join("nas-previews");
        fs::create_dir_all(previews.join("2026/07/Cam")).unwrap();
        fs::write(previews.join("2026/07/Cam/a.CR3.lr.full.jpg"), b"as edited").unwrap();
        run(&mut job, &here, Some(&nas), Some(&previews), &cfg, &fake, &mut |p| last = p.clone()).unwrap();
        assert_eq!(job.problems.len(), 1, "{:?}", job.problems);
        assert!(job.problems[0].contains("b.CR3") && job.problems[0].contains("origineel genomen"));
        assert_eq!(fs::read(dest.join("001-IMG_1.MOV")).unwrap(), b"made before", "kept");
        assert_eq!(fs::read(dest.join("002-a.jpg")).unwrap(), b"as edited", "Lightroom's version");
        assert_eq!(fs::read(dest.join("003-b.jpg")).unwrap(), b"2026/07/Cam/b.CR3", "from the NAS");
        assert_eq!((last.done, last.total), (3, 3));
        let info: serde_json::Value = serde_json::from_slice(&fs::read(dest.join("export.json")).unwrap()).unwrap();
        assert_eq!((info["name"].as_str(), info["from_lists"][0].as_str()), (Some("Aosta"), Some("2026/aosta")));
        assert_eq!(info["lightroom_edits"], serde_json::json!(["002-a.jpg"]));
        assert!(Job::pending(&cfg).is_none());
    }
}
