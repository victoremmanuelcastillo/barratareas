# webbar

Reemplazo completo de la barra de tareas de KDE Plasma en Wayland: en vez de theming/transparencia nativa (limitados por el propio Plasma), es una app de escritorio real construida con **Tauri (Rust) + Tailwind CSS**, anclada a la pantalla vía el protocolo Wayland layer-shell.

## Qué hace

- Se ancla como overlay en el borde inferior de la pantalla (no reserva espacio, no empuja ni redimensiona otras ventanas).
- Autohide: se reduce a una franja de 4px y reaparece al pasar el mouse.
- Fondo semitransparente (no un panel opaco negro).
- Lista de ventanas abiertas con ícono real de la app, click para activar/restaurar.
- Reloj (formato 24h) + fecha, en el mismo espacio.
- Batería (%, vía `upower`).
- Volumen (%, vía `wpctl`/PipeWire).
- Brillo de pantalla (%, vía D-Bus de PowerDevil).
- Botón de portapapeles (abre el historial nativo de Klipper).
- Botón de terminal (abre Konsole).
- Bandeja de sistema (protocolo StatusNotifierItem/DBusMenu — íconos como Arch-Update, etc). Click izquierdo activa, click derecho abre su menú contextual nativo.

## Por qué existe

Plasma no deja personalizar libremente el diseño/transparencia de un panel nativo más allá de cierto punto. Esta es una barra propia, con el control total de HTML/CSS, que igual se integra con el sistema real (KWin, D-Bus, PipeWire) en vez de ser una ventana flotante desconectada del escritorio.

## Requisitos

Pensado y probado en **KDE Plasma 6.x sobre Wayland** (KWin). Depende de herramientas que ya vienen en un escritorio Plasma normal:

- `qdbus6` (KWin scripting, PowerDevil, Klipper)
- `wpctl` (PipeWire — control de volumen)
- `upower` (batería)
- `konsole` (botón de terminal; cambiar en el código si usás otra)

Para compilar:
- Rust + Cargo (`rustup` o el paquete de tu distro)
- Node.js + npm
- Dependencias del sistema para Tauri en Linux (GTK3, WebKitGTK, `gtk-layer-shell`). En Arch/CachyOS:
  ```sh
  sudo pacman -S webkit2gtk-4.1 gtk-layer-shell base-devel curl wget file openssl appmenu-gtk-module libappindicator-gtk3 librsvg
  ```

## Instalar / compilar

```sh
git clone https://github.com/victoremmanuelcastillo/barratareas.git
cd barratareas/webbar
npm install
npm run build:css

cd src-tauri
cargo build --release
```

El binario final queda en `webbar/src-tauri/target/release/app`.

## Correr manualmente

```sh
./webbar/src-tauri/target/release/app
```

## Arranque automático al iniciar sesión

Crear `~/.config/autostart/webbar.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=webbar
Comment=Barra de tareas reemplazo (Tauri + layer-shell)
Exec=/ruta/completa/a/barratareas/webbar/src-tauri/target/release/app
Icon=utilities-terminal
Terminal=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
X-KDE-autostart-phase=1
```

Ajustar `Exec` a la ruta real donde clonaste el repo. Toma efecto en la próxima sesión gráfica (no hace falta reiniciar la máquina).

## Desarrollo / modificar

- `webbar/src/index.html` — frontend (HTML + Tailwind + JS vanilla, sin framework).
- `webbar/src-tauri/src/lib.rs` — backend Rust (comandos Tauri, integración con D-Bus/KWin/GTK).
- `webbar/src-tauri/kwin-scripts/list_windows.js` — script de KWin que reporta la lista de ventanas (KWin no expone el protocolo estándar de gestión de ventanas a clientes normales, así que se usa su API de scripting propia).

**Importante para desarrollo**: Tauri embebe el frontend (`src/`) dentro del binario **al compilar**. Editar `index.html`/CSS y solo reiniciar el binario ya compilado no tiene efecto — hace falta `cargo build` de nuevo (o usar `cargo tauri dev`, que además da hot-reload).

## Desinstalar

```sh
pkill -f "webbar/src-tauri/target/release/app"
rm ~/.config/autostart/webbar.desktop
rm -rf ~/.local/share/cl.victorecn.webbar   # perfil de WebKit (caché, storage)
```

## Estado / pendiente

- `cargo tauri build` genera `.deb` y `.rpm` en `src-tauri/target/release/bundle/` (útil para distribuir a otras distros). El AppImage falla por una herramienta externa (`linuxdeploy`) — no importa para uso propio, en Arch/CachyOS ninguno de los tres formatos aplica igual, se usa el binario suelto (ver "Instalar / compilar" arriba).
- Probado en una sola máquina (CachyOS, Plasma 6.7.3, GPU Intel integrada) — otras distros/compositores no están verificados.
