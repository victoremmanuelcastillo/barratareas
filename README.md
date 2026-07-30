# barratareas

Rediseño de la barra de tareas de Plasma en CachyOS, dividida en dos paneles nuevos que se **agregan** al panel actual sin modificarlo (regla de seguridad: no dañar el sistema principal).

## Diseño acordado (2026-07-30)

- **Panel izquierdo** (abajo): solo el listado de aplicaciones abiertas (`icontasks`). Oculto por defecto, aparece al pasar el mouse (hiding: `autohide`).
- **Panel derecho** (abajo): bandeja del sistema, batería/energía y reloj. Siempre visible, fondo casi transparente.
- El panel único actual (`Containments][23]` en `plasma-org.kde.plasma.desktop-appletsrc`) queda intacto — estos son paneles adicionales, no un reemplazo.

## Estado

- [x] Backup de la config actual del panel → `backups/`
- [x] Script de creación de paneles → `scripts/crear-paneles.js`
- [ ] Aplicar el script (pendiente de confirmación explícita del usuario)
- [ ] Ajustar transparencia del panel derecho (se hace a mano tras crear el panel: click derecho → Entrar en modo de edición → ícono de configuración del panel → Apariencia → Fondo del panel → Adaptativo/Translúcido — no hay una clave de config confiable para setear esto por script sin probarlo en vivo primero)
- [ ] Revisar tamaños/alineación una vez visto en pantalla

## Cómo aplicar (cuando se confirme)

```sh
qdbus6 org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript "$(cat scripts/crear-paneles.js)"
```

## Cómo deshacer

- Quitar un panel nuevo: click derecho sobre él → "Eliminar panel".
- Restaurar la config completa desde el backup:
  ```sh
  cp backups/plasma-org.kde.plasma.desktop-appletsrc.2026-07-30.bak ~/.config/plasma-org.kde.plasma.desktop-appletsrc
  # y reiniciar plasmashell (o cerrar sesión / reiniciar sesión gráfica)
  ```
