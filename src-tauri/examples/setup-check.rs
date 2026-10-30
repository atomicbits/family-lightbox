//! What a first start would find on this Mac (F006/S06), without touching its settings: the NAS
//! adopted into a temporary folder, and the welcome's proposals from the mounted `Lists` share.
//!
//!     cargo run --example setup-check

use family_lightbox_lib::{config, mounts, nas};
use std::path::Path;

fn main() {
    let raw = mounts::system_mounts();
    let tmp = std::env::temp_dir().join(format!("setup-check-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    println!("adopted host: {:?}", config::adopt_host(&tmp, &raw));
    let (settings, _) = config::load_settings(&tmp);
    let found = mounts::find_shares(&raw, &settings);
    let lists = mounts::NasMounts::chosen(&found.lists);
    let user = lists.and_then(|m| m.user.clone());
    let known = lists.map(|m| nas::known_devices(Path::new(&m.mount_point))).unwrap_or_default();
    let owner = user.as_deref().map(nas::owner_from_user).unwrap_or_default();
    let computer = nas::computer_name();
    println!("user {user:?} → owner {owner:?}; computer {computer:?} → device {:?}", nas::propose_device(&computer, &owner, &known));
    println!("known: {known:?}");
    let _ = std::fs::remove_dir_all(&tmp);
}
