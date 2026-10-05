import init, { TourWorld } from "./wasm/cascade_web.js";

const colors = [[23, 33, 43], [120, 131, 141], [141, 78, 49], [229, 184, 79], [240, 88, 50], [69, 169, 197]];
const materials = [["Air", "#17212b"], ["Stone", "#78838d"], ["Wood", "#8d4e31"], ["Sand", "#e5b84f"], ["Explosive", "#f05832"], ["Water", "#45a9c5"]];
const LIMIT = 512;
const RING_CAPACITY = 128;

function makeLegend(root) {
  const legend = document.createElement("ul");
  legend.className = "tour-legend";
  const entries = [
    ...materials.map(([name, color]) => [name, color, "material"]),
    ["pending", "#ffcd4b", "overlay"],
    ["heat / blast", "#ff5c1c", "overlay"],
    ["executed this slice", "#e6eef3", "overlay"],
    ["player-focus chunk (8 slices)", "#5bc0be", "focus"],
  ];
  for (const [name, color, kind] of entries) {
    const item = document.createElement("li");
    const swatch = document.createElement("span");
    swatch.className = `tour-swatch ${kind === "focus" ? "tour-swatch-focus" : ""}`;
    swatch.style.setProperty("--swatch-color", color);
    swatch.setAttribute("aria-hidden", "true");
    item.append(swatch, document.createTextNode(name));
    legend.append(item);
  }
  root.append(legend);
}

function makeControls(root, scene, label, comparison = false) {
  const actions = document.createElement("div");
  actions.className = "tour-controls";
  actions.innerHTML = `<button type="button" data-action="play">Play</button><button type="button" data-action="step">Step one slice</button><button type="button" data-action="reset">Reset</button><label>Credits per slice <input data-action="budget" type="range" min="25" max="${LIMIT}" value="64"><output>64</output></label><label>Paint <select data-action="material"><option value="3">Sand</option><option value="5">Water</option><option value="2">Wood</option><option value="4">Explosive</option><option value="1">Stone</option><option value="0">Erase</option></select></label>${comparison || scene === 4 ? "<button type=\"button\" data-action=\"destroy\">DESTROY PERFORMANCE</button>" : ""}`;
  root.append(actions);
  const slider = actions.querySelector('[data-action="budget"]');
  const output = actions.querySelector("output");
  slider.addEventListener("input", () => { output.value = slider.value; });
  return actions;
}

function makeCanvas(root, title) {
  const figure = document.createElement("figure");
  figure.className = "tour-panel";
  const heading = document.createElement("h3");
  heading.textContent = title;
  const canvas = document.createElement("canvas");
  canvas.className = "tour-canvas";
  canvas.width = 512;
  canvas.height = 512;
  canvas.setAttribute("aria-label", `${title} simulation world, 128 by 128 cells at 4 pixels per cell`);
  const status = document.createElement("p");
  status.className = "tour-stats";
  figure.append(heading, canvas, status);
  root.append(figure);
  const raster = document.createElement("canvas");
  raster.width = 128; raster.height = 128;
  return { canvas, status, context: canvas.getContext("2d", { alpha: false }), raster, rasterContext: raster.getContext("2d") };
}

function draw(view, world, history) {
  const { canvas, context, status } = view;
  const pixels = world.cells();
  const image = context.createImageData(world.width(), world.height());
  const focusedChunks = new Set();
  for (let i = 0, p = 0; i < pixels.length; i += 5, p += 4) {
    const rgb = colors[pixels[i]] || colors[0];
    image.data[p] = rgb[0]; image.data[p + 1] = rgb[1]; image.data[p + 2] = rgb[2]; image.data[p + 3] = 255;
    if (pixels[i + 2]) { image.data[p] = Math.min(255, image.data[p] + 42); image.data[p + 1] = Math.min(255, image.data[p + 1] + 24); }
    if (pixels[i + 1]) { image.data[p] = 255; image.data[p + 1] = Math.max(72, image.data[p + 1] - pixels[i + 1] * 9); image.data[p + 2] = 22; }
    if (pixels[i + 3]) { image.data[p] = Math.min(255, image.data[p] + 90); image.data[p + 1] = Math.min(255, image.data[p + 1] + 90); image.data[p + 2] = Math.min(255, image.data[p + 2] + 90); }
    if (pixels[i + 4]) {
      const cell = p / 4;
      const chunkX = Math.floor((cell % world.width()) / 32);
      const chunkY = Math.floor(Math.floor(cell / world.width()) / 32);
      focusedChunks.add(`${chunkX}:${chunkY}`);
    }
  }
  view.rasterContext.putImageData(image, 0, 0);
  context.imageSmoothingEnabled = false;
  context.drawImage(view.raster, 0, 0, canvas.width, canvas.height);
  context.save(); context.strokeStyle = "#5bc0be"; context.lineWidth = 2; context.setLineDash([6, 4]);
  for (const chunk of focusedChunks) {
    const [x, y] = chunk.split(":").map(Number);
    context.strokeRect(x * 128 + 1, y * 128 + 1, 126, 126);
  }
  context.restore();
  history.push(world.pending() + world.queued_commands()); if (history.length > 100) history.shift();
  const complete = world.pending() === 0 && world.queued_commands() === 0;
  status.textContent = `Slice ${world.slice()} · charged ${world.charged()} / ${world.allowed()} credits · ${world.executed_cells()} quanta ran (${world.evaluations()} eval + ${world.blasts()} blast) · ${world.pending()} pending channels + ${world.queued_commands()} queued commands · ready ${world.ready()}/${RING_CAPACITY} · ${world.focus_regions()} focus regions · oldest pending ${world.oldest_age()} slices · ${complete ? "settled" : "resolving"}`;
}

function holdButton(button, burst) {
  let timer;
  let pointerClick = false;
  const start = () => { if (timer) return; pointerClick = true; burst(); timer = setInterval(burst, 80); };
  const stop = () => { clearInterval(timer); timer = undefined; };
  button.addEventListener("pointerdown", start);
  button.addEventListener("pointerup", stop);
  button.addEventListener("pointerleave", stop);
  button.addEventListener("pointercancel", stop);
  button.addEventListener("click", () => { if (pointerClick) pointerClick = false; else burst(); });
}

function attachPaint(canvas, world, controls, redraw) {
  canvas.addEventListener("click", event => {
    const rect = canvas.getBoundingClientRect();
    const x = Math.floor((event.clientX - rect.left) * world.width() / rect.width);
    const y = Math.floor((event.clientY - rect.top) * world.height() / rect.height);
    if (event.shiftKey) world.paint(x, y, Number(controls.querySelector('[data-action="material"]').value));
    else world.ignite(x, y);
    redraw();
  });
}

function plot(container, values, color, max, logarithmic = false) {
  const canvas = container;
  const ctx = canvas.getContext("2d");
  ctx.strokeStyle = color; ctx.lineWidth = 3; ctx.beginPath();
  const point = value => logarithmic ? Math.log1p(value) / Math.log1p(max) : value / max;
  values.forEach((value, i) => {
    const x = 48 + i * (canvas.width - 60) / Math.max(1, values.length - 1);
    const y = canvas.height - 24 - point(value) * (canvas.height - 44);
    if (!i) ctx.moveTo(x, y); else ctx.lineTo(x, y);
  });
  ctx.stroke();
}

function renderHistory(root, boundedBacklog, traditionalBacklog, boundedCredits, traditionalCredits, allowance) {
  const [backlog, charged] = root.querySelectorAll(".tour-graph");
  const drawChart = canvas => {
    const ctx = canvas.getContext("2d");
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#18212a"; ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.strokeStyle = "#44515c"; ctx.lineWidth = 1;
    for (let row = 1; row <= 3; row++) {
      const y = row * canvas.height / 4;
      ctx.beginPath(); ctx.moveTo(42, y); ctx.lineTo(canvas.width - 8, y); ctx.stroke();
    }
  };
  drawChart(backlog);
  const pendingMax = Math.max(1, ...boundedBacklog, ...traditionalBacklog);
  plot(backlog, boundedBacklog, "#5bc0be", pendingMax);
  plot(backlog, traditionalBacklog, "#ff8966", pendingMax);
  const backlogContext = backlog.getContext("2d");
  backlogContext.fillStyle = "#e6eef3"; backlogContext.font = "12px sans-serif";
  backlogContext.fillText(pendingMax.toLocaleString(), 4, 16); backlogContext.fillText("0", 20, backlog.height - 8); backlogContext.fillText("slice →", backlog.width - 48, backlog.height - 6);
  drawChart(charged);
  const chargedMax = Math.max(1, ...boundedCredits, ...traditionalCredits, allowance);
  plot(charged, boundedCredits, "#5bc0be", chargedMax, true);
  plot(charged, traditionalCredits, "#ff8966", chargedMax, true);
  const ctx = charged.getContext("2d");
  const allowanceY = charged.height - 24 - Math.log1p(allowance) / Math.log1p(chargedMax) * (charged.height - 44);
  ctx.strokeStyle = "#ffcd4b"; ctx.lineWidth = 2; ctx.setLineDash([8, 5]);
  ctx.beginPath(); ctx.moveTo(42, allowanceY); ctx.lineTo(charged.width - 8, allowanceY); ctx.stroke(); ctx.setLineDash([]);
  ctx.fillStyle = "#e6eef3"; ctx.font = "12px sans-serif";
  ctx.fillText(chargedMax.toLocaleString(), 4, 16); ctx.fillText("0", 20, charged.height - 8); ctx.fillText("slice →", charged.width - 48, charged.height - 6);
}

await init();
for (const root of document.querySelectorAll(".tour-widget")) {
  const scene = Number(root.dataset.scene);
  makeLegend(root);
  const controls = makeControls(root, scene, root.dataset.label);
  const view = makeCanvas(root, root.dataset.label);
  const world = new TourWorld(64, false, scene);
  let running = false;
  let timer;
  const history = [];
  const repaint = () => draw(view, world, history);
  controls.querySelector('[data-action="budget"]').addEventListener("input", event => world.set_budget(Number(event.target.value)));
  controls.querySelector('[data-action="step"]').addEventListener("click", () => { world.step(); repaint(); });
  controls.querySelector('[data-action="reset"]').addEventListener("click", () => { world.reset(); repaint(); });
  controls.querySelector('[data-action="play"]').addEventListener("click", event => {
    running = !running; event.target.textContent = running ? "Pause" : "Play";
    if (running) timer = setInterval(() => { world.step(); repaint(); }, 80); else clearInterval(timer);
  });
  if (scene === 4) {
    let disturbance = 0;
    const burst = () => { for (let i = 0; i < 8; i++, disturbance++) world.ignite(32 + (disturbance % 8) * 8, 84); repaint(); };
    holdButton(controls.querySelector('[data-action="destroy"]'), burst);
  }
  attachPaint(view.canvas, world, controls, repaint);
  repaint();
}

for (const root of document.querySelectorAll(".tour-comparison")) {
  const scene = Number(root.dataset.scene);
  makeLegend(root);
  const controls = makeControls(root, scene, root.dataset.label, true);
  const pair = document.createElement("div"); pair.className = "tour-pair"; root.append(pair);
  const boundedView = makeCanvas(pair, "Bounded · credit-limited");
  const traditionalView = makeCanvas(pair, "Traditional · full captured frontier");
  const backlog = document.createElement("canvas"); backlog.width = 640; backlog.height = 190; backlog.className = "tour-graph";
  const backlogTitle = document.createElement("h3"); backlogTitle.textContent = "Outstanding channels + paint commands after each slice";
  const backlogLegend = document.createElement("p"); backlogLegend.className = "tour-stats"; backlogLegend.textContent = "Outstanding pending channels plus queued paint commands: bounded (teal), traditional (coral). Start/stop the same deterministic one-paint-per-slice stream for both schedulers; after stopping it, watch deferred work drain.";
  const charged = document.createElement("canvas"); charged.width = 640; charged.height = 190; charged.className = "tour-graph";
  const chargedTitle = document.createElement("h3"); chargedTitle.textContent = "Credits charged per slice · logarithmic scale";
  const chargedLegend = document.createElement("p"); chargedLegend.className = "tour-stats"; chargedLegend.textContent = "Charge by scheduler: bounded (teal), traditional (coral); dashed amber is the bounded allowance. Values count simulation credits, not milliseconds.";
  root.append(backlogTitle, backlog, backlogLegend, chargedTitle, charged, chargedLegend);
  let budget = 64;
  const bounded = new TourWorld(budget, false, scene);
  const traditional = new TourWorld(budget, true, scene);
  const bh = [], th = [], bc = [], tc = [];
  let stressing = false, disturbance = 0;
  const addLoad = () => {
    const x = 18 + (disturbance * 17 % 92);
    const y = 16 + (disturbance * 11 % 54);
    const material = [1, 2][disturbance % 2];
    disturbance++;
    bounded.paint(x, y, material);
    traditional.paint(x, y, material);
  };
  const redraw = () => {
    draw(boundedView, bounded, bh);
    draw(traditionalView, traditional, th);
    bc.push(bounded.charged()); tc.push(traditional.charged());
    if (bc.length > 100) { bc.shift(); tc.shift(); }
    renderHistory(root, bh, th, bc, tc, budget);
  };
  controls.querySelector('[data-action="budget"]').addEventListener("input", e => { budget = Number(e.target.value); bounded.set_budget(budget); traditional.set_budget(budget); renderHistory(root, bh, th, bc, tc, budget); });
  controls.querySelector('[data-action="step"]').addEventListener("click", () => { if (stressing) addLoad(); bounded.step(); traditional.step(); redraw(); });
  controls.querySelector('[data-action="reset"]').addEventListener("click", () => { bounded.reset(); traditional.reset(); stressing = false; disturbance = 0; controls.querySelector('[data-action="destroy"]').textContent = "DESTROY PERFORMANCE"; bh.length = 0; th.length = 0; bc.length = 0; tc.length = 0; redraw(); });
  let running = false, timer;
  controls.querySelector('[data-action="play"]').addEventListener("click", e => { running = !running; e.target.textContent = running ? "Pause" : "Play"; if (running) timer = setInterval(() => { if (stressing) addLoad(); bounded.step(); traditional.step(); redraw(); }, 80); else clearInterval(timer); });
  controls.querySelector('[data-action="destroy"]').addEventListener("click", e => { stressing = !stressing; e.currentTarget.textContent = stressing ? "Stop disturbance" : "DESTROY PERFORMANCE"; });
  attachPaint(boundedView.canvas, bounded, controls, redraw);
  attachPaint(traditionalView.canvas, traditional, controls, redraw);
  redraw();
}
