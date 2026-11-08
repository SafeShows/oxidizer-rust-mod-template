//! my_mod: an Oxidizer mod written in Rust. API docs: https://docs.rs/oxidizer_api
//!
//! This uses the by-name API (`Obj::static_get("KSA.Program, KSA", ...)`). For typed access with autocomplete,
//! run `oxidizer --bindgen src/ksa.rs` and use the generated types instead (see README).

use oxidizer_api::serde::{Deserialize, Serialize};
use oxidizer_api::{Event, Mod, Obj, Subscription, imgui, key_pressed, log};

/// An event other mods (Rust or Rhai) can subscribe to. Topics start with your mod's id: only my_mod can publish
/// on `my_mod.*`. Rhai mods get it as a map: `#{ count: 3 }`.
#[derive(Serialize, Deserialize)]
#[serde(crate = "oxidizer_api::serde")]
struct Clicked {
    count: u32,
}
impl Event for Clicked {
    const TOPIC: &'static str = "my_mod.clicked";
}

struct MyMod {
    visible: bool,
    clicks: u32,
    /// Kept while the mod runs: dropping it unsubscribes.
    _on_click: Option<Subscription>,
}

impl Mod for MyMod {
    fn init() -> Self {
        log("my_mod: hello!"); // shows in the log and in Mods > Oxidizer

        // Example hook: runs after the game's Program.OnFocus. `args` are the method's arguments; return
        // false from a prefix (postfix = false) to skip the original method.
        oxidizer_api::hook("KSA.Program", "OnFocus", true, |_program, args| {
            log(&format!("my_mod: focus changed: {}", args.index(1)));
            true
        });

        // Example subscription, the way another mod would listen. Events arrive at the start of the next frame.
        let on_click = oxidizer_api::subscribe(|c: Clicked| log(&format!("my_mod: clicked {} times", c.count))).ok();

        MyMod { visible: true, clicks: 0, _on_click: on_click }
    }

    /// Once the game has loaded its content and mods (init runs earlier, before the game's Main). Set up what
    /// needs the game's data here; there is no flight yet, so the controlled vehicle comes later.
    fn loaded(&mut self) {
        log("my_mod: the game has loaded");
    }

    /// Every frame. key_pressed is true only on the frame the key went down.
    fn frame(&mut self, _time: f64, _dt: f64) {
        if key_pressed("F8") {
            self.visible = !self.visible;
        }
    }

    /// Your entries in the game's Mods menu.
    fn menu(&mut self) {
        if imgui::menu_item("My mod", self.visible) {
            self.visible = !self.visible;
        }
    }

    /// Your windows (ImGui). Always call end() after begin().
    fn gui(&mut self, _dt: f64) {
        if !self.visible {
            return;
        }
        if imgui::begin("My mod") {
            let vehicle = Obj::static_get("KSA.Program, KSA", "ControlledVehicle");
            if vehicle.is_null() {
                imgui::text("no vehicle");
            } else {
                imgui::text(&format!("vehicle: {}", vehicle.get("Id")));
            }
            let time = Obj::static_call("KSA.Universe, KSA", "GetElapsedSeconds", &[]).f64().unwrap_or(0.0);
            imgui::text(&format!("time: {time:.1} s"));
            if imgui::button("Click me") {
                self.clicks += 1;
                let _ = oxidizer_api::publish(&Clicked { count: self.clicks });
            }
            imgui::same_line();
            imgui::text(&format!("clicks: {}", self.clicks));
            imgui::text("F8 hides this window");
        }
        imgui::end();
    }
}

oxidizer_api::oxidizer_mod!(MyMod);
