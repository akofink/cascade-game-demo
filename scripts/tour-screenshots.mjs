import { chromium } from "playwright-core";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";

const base = process.env.TOUR_URL || "http://127.0.0.1:3000/docs/tour.html";
const executablePath = process.env.CHROME_PATH || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const output = resolve("docs/tour/images");
const capture = process.env.CAPTURE !== "0";
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true, executablePath, args: ["--no-sandbox"] });
const page = await browser.newPage({ viewport: { width: 1280, height: 1100 }, deviceScaleFactor: 1 });
const failures = [];
page.on("pageerror", error => failures.push(error.message));
page.on("response", response => { if (response.status() >= 400) failures.push(`${response.status()} ${response.url()}`); });
await page.goto(base, { waitUntil: "networkidle" });
await page.waitForFunction(() => document.querySelectorAll(".tour-widget canvas").length === 4 && document.querySelector(".tour-comparison canvas"));

const widgets = await page.locator(".tour-widget").all();
const names = ["cells", "credits", "deferral", "destroy-performance"];
for (let i = 0; i < widgets.length; i++) {
  const widget = widgets[i];
  const start = await widget.locator(".tour-stats").textContent();
  await widget.locator('[data-action="budget"]').fill("64");
  await widget.locator('[data-action="material"]').selectOption("3");
  await widget.locator("canvas").click({ position: { x: 256, y: 256 }, modifiers: ["Shift"] });
  for (let step = 0; step < 5; step++) await widget.locator('[data-action="step"]').click();
  const end = await widget.locator(".tour-stats").textContent();
  if (start === end || !end.includes("slice")) throw new Error(`Widget ${i} did not advance`);
  if (i === 3) await widget.locator('[data-action="destroy"]').click();
  if (capture) await widget.screenshot({ path: resolve(output, `${names[i]}.png`), animations: "disabled" });
}
const comparison = page.locator(".tour-comparison");
await comparison.locator('[data-action="budget"]').fill("64");
await comparison.locator('[data-action="destroy"]').click();
for (let step = 0; step < 6; step++) await comparison.locator('[data-action="step"]').click();
if (capture) {
  await comparison.screenshot({ path: resolve(output, "comparison.png"), animations: "disabled" });
  await page.locator(".tour-placeholder").screenshot({ path: resolve(output, "player-focus.png") });
}
await page.locator("img").evaluateAll(images => Promise.all(images.map(image => { image.loading = "eager"; return image.decode(); })));
const brokenImages = await page.locator("img").evaluateAll(images => images.filter(image => !image.complete || image.naturalWidth === 0).map(image => image.src));
if (brokenImages.length) failures.push(`Broken images: ${brokenImages.join(", ")}`);

for (const widget of widgets) {
  await widget.locator('[data-action="reset"]').click();
  await widget.locator('[data-action="play"]').click();
  await page.waitForTimeout(180);
  await widget.locator('[data-action="play"]').click();
}
if (failures.length) throw new Error(`Browser errors:\n${failures.join("\n")}`);
await browser.close();
console.log(`${capture ? `Captured ${names.length + 2} feature screenshots from` : "Verified widgets on"} ${base}`);
