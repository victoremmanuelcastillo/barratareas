function emit() {
  var list = [];
  var clients = workspace.windowList();
  for (var i = 0; i < clients.length; i++) {
    var c = clients[i];
    if (!c.normalWindow) continue;
    if (c.resourceClass === "app") continue;
    list.push({
      caption: c.caption,
      resourceClass: c.resourceClass,
      minimized: c.minimized,
      active: c.active
    });
  }
  print("WEBBAR_WINDOWS:" + JSON.stringify(list));
}
workspace.windowAdded.connect(emit);
workspace.windowRemoved.connect(emit);
workspace.windowActivated.connect(emit);
emit();
