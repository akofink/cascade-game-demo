import init, { TourWorld } from "./wasm/cascade_web.js";

const colors = [[23, 33, 43], [120, 131, 141], [141, 78, 49], [229, 184, 79], [240, 88, 50], [69, 169, 197]];
const LIMIT = 512;
const RING_CAPACITY = 128;

function makeControls(root, scene, label, comparison = false) {
  const actions = document.createElement("div");
  actions.className = "tour-controls";
  actions.innerHTML = `<button type="button" data-action="play">Play</button><button type="button" data-action="step">Step one slice</button><button type="button" data-action="reset">Reset</button><label>Credits <input data-action="budget" type="range" min="25" max="${LIMIT}" value="256"><output>256</output></label><label>Paint <select data-action="material"><option value="3">Sand</option><option value="5">Water</option><option value="2">Wood</option><option value="4">Explosive</option><option value="1">Stone</option><option value="0">Erase</option></select></label>${comparison || scene === 4 ? "<button type=\"button\" data-action=\"destroy\">DESTROY PERFORMANCE</button>" : ""}`;
  root.append(actions);
  const slider = actions.querySelector('[data-action="budget"]');
  const output = actions.querySelector("output");
  slider.addEventListener("input", () => { output.value = slider.value; });
  return actions;
}

function makeCanvas(root, title, className = "tour-canvas") {
  const figure = document.createElement("figure");
  figure.className = "tour-panel";
  const heading = document.createElement("h3");
  heading.textContent = title;
  const canvas = document.createElement("canvas");
  canvas.className = className;
  canvas.width = 512;
  canvas.height = 512;
  canvas.setAttribute("aria-label", `${title} simulation world`);
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
  for (let i = 0, p = 0; i < pixels.length; i += 4, p += 4) {
    const rgb = colors[pixels[i]] || colors[0];
    image.data[p] = rgb[0]; image.data[p + 1] = rgb[1]; image.data[p + 2] = rgb[2]; image.data[p + 3] = 255;
    if (pixels[i + 2]) { image.data[p] = Math.min(255, image.data[p] + 42); image.data[p + 1] = Math.min(255, image.data[p + 1] + 24); }
    if (pixels[i + 1] && pixels[i] === 2) { image.data[p] = 255; image.data[p + 1] = Math.max(72, image.data[p + 1] - pixels[i + 1] * 9); image.data[p + 2] = 22; }
    else if (pixels[i + 1]) { image.data[p] = 255; image.data[p + 1] = Math.max(72, image.data[p + 1] - pixels[i + 1] * 9); image.data[p + 2] = 22; }
    if (pixels[i + 3]) { image.data[p] = Math.min(255, image.data[p] + 90); image.data[p + 1] = Math.min(255, image.data[p + 1] + 90); image.data[p + 2] = Math.min(255, image.data[p + 2] + 90); }
  }
  view.rasterContext.putImageData(image, 0, 0);
  context.imageSmoothingEnabled = false;
  context.drawImage(view.raster, 0, 0, canvas.width, canvas.height);
  history.push(world.pending()); if (history.length > 100) history.shift();
  const complete = world.pending() === 0 && world.queued_commands() === 0;
  status.textContent = `slice ${world.slice()} · ${world.evaluations()} eval + ${world.blasts()} blast quanta (${world.executed_cells_sampled()} cells highlighted / ${world.executed_cells()} executed) · ${world.charged()}/${world.allowed()} credits · ready ${world.ready()}/${RING_CAPACITY} · ${world.recoveries()} recoveries · ${world.pending()} pending · oldest ${world.oldest_age()} slices · ${complete ? "settled" : `${world.queued_commands()} commands / resolving`} · ${world.slice_time_ms().toFixed(3)} ms (browser illustrative)`;
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

function renderHistory(container, bounded, traditional) {
  const canvas = container.querySelector(".tour-graph");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#18212a"; ctx.fillRect(0, 0, canvas.width, canvas.height);
  const max = Math.max(1, ...bounded, ...traditional);
  for (const [values, color] of [[bounded, "#5bc0be"], [traditional, "#ff8966"]]) {
    ctx.strokeStyle = color; ctx.lineWidth = 2; ctx.beginPath();
    values.forEach((value, i) => { const x = i * canvas.width / Math.max(1, values.length - 1); const y = canvas.height - value * (canvas.height - 10) / max; if (!i) ctx.moveTo(x, y); else ctx.lineTo(x, y); });
    ctx.stroke();
  }
}

await init();
for (const root of document.querySelectorAll(".tour-widget")) {
  const scene = Number(root.dataset.scene);
  const controls = makeControls(root, scene, root.dataset.label);
  const view = makeCanvas(root, root.dataset.label);
  const world = new TourWorld(256, false, scene);
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
  const controls = makeControls(root, scene, root.dataset.label, true);
  const pair = document.createElement("div"); pair.className = "tour-pair"; root.append(pair);
  const boundedView = makeCanvas(pair, "Bounded · budgeted slices");
  const traditionalView = makeCanvas(pair, "Traditional · full captured frontier");
  const graph = document.createElement("canvas"); graph.width = 640; graph.height = 160; graph.className = "tour-graph";
  const legend = document.createElement("p"); legend.className = "tour-stats"; legend.textContent = "Backlog over time · bounded (teal), traditional (coral)";
  root.append(graph, legend);
  let budget = 256;
  const bounded = new TourWorld(budget, false, scene);
  const traditional = new TourWorld(budget, true, scene);
  const bh = [], th = [];
  const redraw = () => { draw(boundedView, bounded, bh); draw(traditionalView, traditional, th); renderHistory(root, bh, th); };
  controls.querySelector('[data-action="budget"]').addEventListener("input", e => { budget = Number(e.target.value); bounded.set_budget(budget); traditional.set_budget(budget); });
  controls.querySelector('[data-action="step"]').addEventListener("click", () => { bounded.step(); traditional.step(); redraw(); });
  controls.querySelector('[data-action="reset"]').addEventListener("click", () => { bounded.reset(); traditional.reset(); bh.length = 0; th.length = 0; redraw(); });
  let running = false, timer;
  controls.querySelector('[data-action="play"]').addEventListener("click", e => { running = !running; e.target.textContent = running ? "Pause" : "Play"; if (running) timer = setInterval(() => { bounded.step(); traditional.step(); redraw(); }, 80); else clearInterval(timer); });
  holdButton(controls.querySelector('[data-action="destroy"]'), () => { bounded.ignite(32, 78); traditional.ignite(32, 78); redraw(); });
  attachPaint(boundedView.canvas, bounded, controls, redraw);
  attachPaint(traditionalView.canvas, traditional, controls, redraw);
  redraw();
}
