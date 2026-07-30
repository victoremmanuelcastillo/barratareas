// Script de Plasma Scripting API (KF6 / Plasma 6.7.3)
// Objetivo: AGREGAR dos paneles nuevos, sin tocar el panel actual (Containments][23]).
// Se ejecuta con:
//   qdbus6 org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript "$(cat scripts/crear-paneles.js)"
//
// Panel izquierdo: solo apps abiertas, oculto hasta pasar el mouse.
var panelIzquierdo = new Panel;
panelIzquierdo.location = 'bottom';
panelIzquierdo.alignment = 'left';
panelIzquierdo.height = 44;
panelIzquierdo.hiding = 'autohide';
panelIzquierdo.addWidget('org.kde.plasma.icontasks');

// Panel derecho: reloj + batería/energía + bandeja del sistema, siempre visible.
var panelDerecho = new Panel;
panelDerecho.location = 'bottom';
panelDerecho.alignment = 'right';
panelDerecho.height = 36;
panelDerecho.hiding = 'normal';
panelDerecho.addWidget('org.kde.plasma.systemtray');
panelDerecho.addWidget('org.kde.plasma.battery');
panelDerecho.addWidget('org.kde.plasma.digitalclock');
