use tauri::{Emitter, Manager};

const LIST_WINDOWS_SCRIPT: &str = include_str!("../kwin-scripts/list_windows.js");

fn run_kwin_script(path: &std::path::Path) -> Result<(), String> {
  let load = std::process::Command::new("qdbus6")
    .args(["org.kde.KWin", "/Scripting", "loadScript", path.to_str().ok_or("bad path")?])
    .output()
    .map_err(|e| e.to_string())?;
  let script_id = String::from_utf8_lossy(&load.stdout).trim().to_string();
  std::process::Command::new("qdbus6")
    .args(["org.kde.KWin", &format!("/Scripting/Script{script_id}"), "run"])
    .output()
    .map_err(|e| e.to_string())?;
  Ok(())
}

fn spawn_window_watcher(app_handle: tauri::AppHandle) {
  std::thread::spawn(move || {
    let child = std::process::Command::new("journalctl")
      .args(["-f", "-n", "0", "-o", "cat"])
      .stdout(std::process::Stdio::piped())
      .spawn();

    let mut child = match child {
      Ok(c) => c,
      Err(e) => {
        eprintln!("journalctl spawn failed: {e}");
        return;
      }
    };

    if let Some(stdout) = child.stdout.take() {
      use std::io::BufRead;
      let reader = std::io::BufReader::new(stdout);
      for line in reader.lines().flatten() {
        if let Some(json_str) = line.strip_prefix("WEBBAR_WINDOWS:") {
          if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
            let _ = app_handle.emit("windows-updated", value);
          }
        }
      }
    }
  });
}

#[tauri::command]
fn activate_window(resource_class: String, caption: String) -> Result<(), String> {
  let script = format!(
    "var clients = workspace.windowList();\nfor (var i=0;i<clients.length;i++){{var c=clients[i]; if(c.resourceClass==={} && c.caption==={}){{ if(c.minimized){{c.minimized=false;}} workspace.activeWindow=c; break; }} }}",
    serde_json::to_string(&resource_class).map_err(|e| e.to_string())?,
    serde_json::to_string(&caption).map_err(|e| e.to_string())?
  );
  let path = std::env::temp_dir().join("webbar-activate.js");
  std::fs::write(&path, script).map_err(|e| e.to_string())?;
  run_kwin_script(&path)
}

#[tauri::command]
fn get_battery() -> Result<serde_json::Value, String> {
  let devices = std::process::Command::new("upower")
    .arg("-e")
    .output()
    .map_err(|e| e.to_string())?;
  let devices = String::from_utf8_lossy(&devices.stdout);
  let battery_path = devices
    .lines()
    .find(|l| l.contains("battery_"))
    .ok_or("no hay bateria")?;

  let info = std::process::Command::new("upower")
    .args(["-i", battery_path])
    .output()
    .map_err(|e| e.to_string())?;
  let info = String::from_utf8_lossy(&info.stdout);

  let mut percentage: u8 = 0;
  let mut state = String::from("unknown");
  for line in info.lines() {
    let line = line.trim();
    if let Some(v) = line.strip_prefix("percentage:") {
      percentage = v.trim().trim_end_matches('%').parse().unwrap_or(0);
    } else if let Some(v) = line.strip_prefix("state:") {
      state = v.trim().to_string();
    }
  }

  Ok(serde_json::json!({ "percentage": percentage, "state": state }))
}

#[tauri::command]
fn get_volume() -> Result<serde_json::Value, String> {
  let out = std::process::Command::new("wpctl")
    .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
    .output()
    .map_err(|e| e.to_string())?;
  let text = String::from_utf8_lossy(&out.stdout);
  let muted = text.contains("[MUTED]");
  let percentage = text
    .split_whitespace()
    .nth(1)
    .and_then(|v| v.parse::<f32>().ok())
    .map(|v| (v * 100.0).round() as i32)
    .unwrap_or(0);
  Ok(serde_json::json!({ "percentage": percentage, "muted": muted }))
}

fn powerdevil_brightness(method: &str) -> Result<i64, String> {
  let out = std::process::Command::new("qdbus6")
    .args([
      "org.kde.Solid.PowerManagement",
      "/org/kde/Solid/PowerManagement/Actions/BrightnessControl",
      &format!("org.kde.Solid.PowerManagement.Actions.BrightnessControl.{method}"),
    ])
    .output()
    .map_err(|e| e.to_string())?;
  String::from_utf8_lossy(&out.stdout).trim().parse::<i64>().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_brightness() -> Result<serde_json::Value, String> {
  let current = powerdevil_brightness("brightness")?;
  let max = powerdevil_brightness("brightnessMax")?;
  let percentage = if max > 0 { (current * 100 / max) as i32 } else { 0 };
  Ok(serde_json::json!({ "percentage": percentage }))
}

#[tauri::command]
fn open_clipboard_menu() -> Result<(), String> {
  std::process::Command::new("qdbus6")
    .args(["org.kde.klipper", "/klipper", "org.kde.klipper.klipper.showKlipperPopupMenu"])
    .output()
    .map_err(|e| e.to_string())?;
  Ok(())
}

fn tray_snapshot_list(client: &system_tray::client::Client) -> Vec<serde_json::Value> {
  let items = client.items();
  let items = items.lock().expect("tray items lock envenenado");
  items
    .iter()
    .map(|(address, (item, _menu))| {
      serde_json::json!({
        "address": address,
        "title": item.title.clone().unwrap_or_else(|| item.id.clone()),
        "iconName": item.icon_name,
      })
    })
    .collect()
}

fn emit_tray_snapshot(app_handle: &tauri::AppHandle, client: &system_tray::client::Client) {
  let _ = app_handle.emit("tray-updated", tray_snapshot_list(client));
}

#[tauri::command]
fn get_tray_items(
  client: tauri::State<'_, std::sync::Arc<system_tray::client::Client>>,
) -> Vec<serde_json::Value> {
  tray_snapshot_list(&client)
}

fn spawn_tray_watcher(app_handle: tauri::AppHandle) {
  tauri::async_runtime::spawn(async move {
    let client = match system_tray::client::Client::new().await {
      Ok(c) => std::sync::Arc::new(c),
      Err(e) => {
        log::error!("no se pudo iniciar el cliente de bandeja: {e}");
        return;
      }
    };

    app_handle.manage(client.clone());

    let mut rx = client.subscribe();
    emit_tray_snapshot(&app_handle, &client);

    while rx.recv().await.is_ok() {
      emit_tray_snapshot(&app_handle, &client);
    }
  });
}

#[tauri::command]
async fn activate_tray_item(
  client: tauri::State<'_, std::sync::Arc<system_tray::client::Client>>,
  address: String,
) -> Result<(), String> {
  client
    .activate(system_tray::client::ActivateRequest::Default { address, x: 0, y: 0 })
    .await
    .map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
#[tauri::command]
async fn get_icon(app: tauri::AppHandle, icon_name: String) -> Result<Option<String>, String> {
  let (tx, rx) = tokio::sync::oneshot::channel();

  app
    .run_on_main_thread(move || {
      let result = (|| -> Option<String> {
        use base64::Engine;
        use gtk::prelude::IconThemeExt;

        let theme = gtk::IconTheme::default()?;
        let icon_info = theme.lookup_icon(&icon_name, 32, gtk::IconLookupFlags::empty())?;
        let pixbuf = icon_info.load_icon().ok()?;
        let bytes = pixbuf.save_to_bufferv("png", &[]).ok()?;
        Some(format!(
          "data:image/png;base64,{}",
          base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
      })();
      let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;

  rx.await.map_err(|e| e.to_string())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
async fn get_icon(_icon_name: String) -> Result<Option<String>, String> {
  Ok(None)
}

#[cfg(target_os = "linux")]
fn anchor_as_panel(window: &tauri::WebviewWindow) {
  use gtk::glib::Propagation;
  use gtk::prelude::{WidgetExt, WidgetExtManual};
  use gtk_layer_shell::{Edge, Layer, LayerShell};
  use tauri::Emitter;

  let gtk_window = window.gtk_window().expect("no gtk window");

  gtk_window.init_layer_shell();
  gtk_window.set_layer(Layer::Top);
  gtk_window.set_anchor(Edge::Left, true);
  gtk_window.set_anchor(Edge::Right, true);
  gtk_window.set_anchor(Edge::Bottom, true);
  gtk_window.set_namespace("webbar");

  const SHOWN_HEIGHT: i32 = 48;
  const HIDDEN_HEIGHT: i32 = 4;

  // 0 = overlay: no reserva espacio, no empuja ni redimensiona otras ventanas.
  gtk_window.set_exclusive_zone(0);
  gtk_window.set_size_request(1920, HIDDEN_HEIGHT);

  gtk_window.add_events(gtk::gdk::EventMask::ENTER_NOTIFY_MASK | gtk::gdk::EventMask::LEAVE_NOTIFY_MASK);

  let gw = gtk_window.clone();
  let win = window.clone();
  gtk_window.connect_enter_notify_event(move |_, _| {
    gw.set_size_request(1920, SHOWN_HEIGHT);
    let _ = win.emit("hover", true);
    Propagation::Proceed
  });

  let gw = gtk_window.clone();
  let win = window.clone();
  gtk_window.connect_leave_notify_event(move |_, _| {
    gw.set_size_request(1920, HIDDEN_HEIGHT);
    let _ = win.emit("hover", false);
    Propagation::Proceed
  });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let window = app.get_webview_window("main").expect("no main window");

      #[cfg(target_os = "linux")]
      anchor_as_panel(&window);

      window.show()?;

      let watcher_script = std::env::temp_dir().join("webbar-list-windows.js");
      std::fs::write(&watcher_script, LIST_WINDOWS_SCRIPT)?;
      if let Err(e) = run_kwin_script(&watcher_script) {
        log::error!("no se pudo cargar el kwin script de ventanas: {e}");
      }
      spawn_window_watcher(app.handle().clone());
      spawn_tray_watcher(app.handle().clone());

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_battery,
      get_volume,
      get_brightness,
      open_clipboard_menu,
      activate_window,
      activate_tray_item,
      get_icon,
      get_tray_items
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
