import init, {
  builtinPatternSetNames,
  builtinPatternSet,
  compilePatternText,
  findKashidaPoints,
  version,
} from "./pkg/kashida.js";

const TATWEEL = "ـ";

const SAMPLES = {
  Arabic:
    "قال أفلاطون: «الخط عقال العقل». وقال إقليدس " +
    "الإغريقي: «الخط هندسة روحانية وإن ظهرت بآلة " +
    "جسمانية». وقال أبو دلف رحالة القرن العاشر " +
    "الميلادي: «الخط رياض العلوم». وقال النظام المعتزل: " +
    "«الخط أصيل في الروح وإن ظهر بحواس البدن». " +
    "ويورد ابن النديم في الفهرست: «لم يكن اليونانيون " +
    "يعرفون الخط في القديم حتى ورد رجلان من مصر " +
    "يسمى أحدهما قيمس والآخر أغنور. ومعهما ستة " +
    "عشر حرفا فكتب بها اليونان. ثم استنبط أحدهما " +
    "أربعة أحرف فكتب بها ثم استنبط آخر أربعة " +
    "فصارت أربعة وعشرين».",
  Syriac:
    "ܩܘ݂ܝܵܡܵܐ ܕܟܠ ܚܕܵܐ ܐܘ݂ܡܬܵܐ ܬܸܠܝܵܐ ܝܠܹܗ " +
    "ܒܠܸܫܵܢܘ݁ܗ، ܘܠܸܫܵܢܵܐ ܒܟܬܝ݂ܵܒ݂ܵܬܘ݂ܗܝ " +
    "ܘܒܣܸܦܪܵܝܘ݂ܬܘ݂ܗܝ. ܚܲܕ ܠܸܫܵܢܵܐ ܕܠܐ ܟܬܝܼ̈ܒܹܬܵܐ، " +
    "ܐܲܝܟ ܚܲܕ ܟܲܪܡܵܐ ܝܠܹܗ ܕܠܵܐ ܢܵܛܘܿܪܹ̈ܐ. ܐܵܗܵܐ ܒܸܬ " +
    "ܦܵܐܹܣ ܐ݇ܟ݂ܝܼܠܵܐ ܒܓܸܠܹ̈ܐ ܫܹܐܕܵܢܹ̈ܐ، ܘܠܸܫܵܢܵܐ " +
    "ܒܚܵܒܪܹ̈ܐ ܢܘ݂ܼܟܪ݂̈ܵܝܹܐ.",
};

const GOOGLE_FONTS = {
  Arabic: [
    "Amiri", "Alexandria", "Alkalami", "Almarai", "Alyamama", "Amiri Quran",
    "Badeen Display", "Baloo Bhaijaan 2", "Beiruti", "Cairo", "Cairo Play",
    "Cascadia Code", "Cascadia Mono", "Changa", "El Messiri", "Estedad",
    "Fustat", "Handjet", "Harmattan", "IBM Plex Sans Arabic", "Jomhuria",
    "Katibeh", "Kufam", "Lalezar", "Lateef", "Lemonada", "Mada", "Marhey",
    "Markazi Text", "Noto Kufi Arabic", "Noto Naskh Arabic",
    "Noto Sans Arabic", "Oi", "Parastoo", "Playpen Sans Arabic", "Qahiri",
    "Rakkas", "Readex Pro", "Reem Kufi", "Reem Kufi Fun", "Reem Kufi Ink",
    "Rubik", "Ruwudu", "Scheherazade New", "Tajawal", "Vazirmatn", "Vibes",
    "Zain",
  ],
  Syriac: [
    "Idiqlat", "Noto Sans Syriac", "Noto Sans Syriac Eastern",
    "Noto Sans Syriac Western", "Ramsina",
  ],
};

const esc = (text) =>
  text.replace(/[&<>]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" })[c]);

const ctx = document.createElement("canvas").getContext("2d");
const arabic = document.documentElement.lang === "ar";

const numbers = new Intl.NumberFormat(arabic ? "ar-u-nu-arab" : "en", {
  useGrouping: false,
});
const num = numbers.format;

const decimal = numbers.formatToParts(0.1).find((part) => part.type === "decimal").value;
const versionNumber = (value) => value.split(".").map(num).join(decimal);

const plural = new Intl.PluralRules(arabic ? "ar" : "en");

const counted = (value, forms) => forms[plural.select(value)].replace("{}", num(value));

const TEXT = arabic
  ? {
    failed: (name) => `تعذر تحميل ${name}`,
    compiled: "صُرِّفت",
    none: "لا مواضع كشيدة.",
    version: (number) => `(الإصدار ${versionNumber(number)})`,
    px: (value) => `${value} بكسل`,
    head: ["الأولوية", "الموضع", "المعاينة"],
    line: {
      zero: "لا أسطر",
      one: "سطر واحد",
      two: "سطران",
      few: "{} أسطر",
      many: "{} سطرًا",
      other: "{} سطر",
    },
    kashida: {
      zero: "لا كشائد",
      one: "كشيدة واحدة",
      two: "كشيدتان",
      few: "{} كشائد",
      many: "{} كشيدة",
      other: "{} كشيدة",
    },
    lines: (count) => counted(count, TEXT.line),
    stat: (count, kashidas) =>
      `${counted(count, TEXT.line)}، ${counted(kashidas, TEXT.kashida)}`,
  }
  : {
    failed: (name) => `Could not load ${name}`,
    compiled: "compiled",
    none: "No kashida points.",
    version: (number) => `(version ${versionNumber(number)})`,
    px: (value) => `${value}px`,
    head: ["Priority", "Index", "Preview"],
    line: { one: "{} line", other: "{} lines" },
    kashida: { one: "{} kashida", other: "{} kashidas" },
    lines: (count) => counted(count, TEXT.line),
    stat: (count, kashidas) =>
      `${counted(count, TEXT.line)}, ${counted(kashidas, TEXT.kashida)}`,
  };

const state = { family: null, set: null, syriac: false, faces: 0, text: SAMPLES.Arabic };

const segmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" });
const clustersOf = (text) => Array.from(segmenter.segment(text), (part) => part.segment);
const findPoints = (word) => findKashidaPoints(word, state.set, !document.getElementById("keep").checked);

function elongated(word) {
  let out = word.text;
  for (const point of [...word.points].reverse()) {
    out = out.slice(0, point.offset) + TATWEEL.repeat(point.count) + out.slice(point.offset);
  }
  return out;
}

function elongatedWidth(words) {
  return ctx.measureText(words.map((word) => elongated(word)).join(" ")).width;
}

function breakLines(words, width) {
  const lines = [];
  let line = [];
  for (const word of words) {
    if (line.length && elongatedWidth(line.concat(word)) > width) {
      lines.push(line);
      line = [word];
    } else {
      line.push(word);
    }
  }
  if (line.length) lines.push(line);
  return lines;
}

// Insert at least `min` kashidas and at most `max` kashidas at `point`, as long
// as the new width does not exceed the requested one, or return false.
function insert(words, point, width, min, max) {
  const before = point.count;
  if (before >= max) return false;

  point.count = before === 0 ? Math.min(min, max) : before + 1;
  if (elongatedWidth(words) <= width) return true;
  point.count = before;
  return false;
}

// Fills a line with kashidas, stopping short of `width` so the spaces absorb what is left.
function justifyArabic(words, width, { min, max }) {
  const short = () => elongatedWidth(words) < width;
  const insertAt = (point) => insert(words, point, width, min, max);

  const byPriority = Map.groupBy(words.flatMap((word) => word.points), (point) => point.priority);
  const taken = new Set();
  const fill = (point) => { while (insertAt(point)) taken.add(point.word); };

  for (const priority of [...byPriority.keys()].sort((a, b) => b - a)) {
    if (!short()) break;

    // One point per word, the one nearest the word's end.
    const perWord = new Map();
    for (const point of byPriority.get(priority)) {
      if (taken.has(point.word)) continue;
      const current = perWord.get(point.word);
      if (!current || point.offset > current.offset) perWord.set(point.word, point);
    }

    for (const point of [...perWord.values()].reverse()) {
      if (!short()) break;
      fill(point);
    }
  }

  // Still short: add more kashidas to the slots already chosen, up to `max`.
  const slots = words.flatMap((word) => word.points).filter((point) => point.count).reverse();
  fillSlots(words, slots, width, min, max);
}

// Insert one kashida at each slot in turn until the line is full or no slot
// can take another.
function fillSlots(words, slots, width, min, max) {
  const short = () => elongatedWidth(words) < width;
  while (short()) {
    let progressed = false;
    for (const point of slots) {
      if (!short()) break;
      if (insert(words, point, width, min, max)) progressed = true;
    }
    if (!progressed) break;
  }
}

// The Syriac guidelines ask for a different strategy: spread the kashidas
// evenly over every word in the line, one at a time at each word's strongest
// point, rather than one word per priority skipping its neighbors.
function justifySyriac(words, width, { min, max }) {
  const slots = words
    .filter((word) => word.points.length)
    .map((word) => word.points.reduce((a, b) => (a.priority >= b.priority ? a : b)));
  fillSlots(words, slots, width, min, max);
}

function layout({ text, width, min, max, justified, kashida, syriac }) {
  const words = text
    .split(/\s+/)
    .filter(Boolean)
    .map((word, index) => {
      const [text, found] = findPoints(word);
      const clusters = clustersOf(text);
      // Map the point's cluster index to a string offset.
      const points = found.map((point) => ({
        offset: clusters.slice(0, point.index + 1).join("").length,
        priority: point.priority,
        word: index,
        count: 0,
      }));
      return { text, points };
    });

  const lines = breakLines(words, width);
  const justify = syriac ? justifySyriac : justifyArabic;
  return lines.map((line, index) => {
    const stretch = justified && index < lines.length - 1;
    if (stretch && kashida) justify(line, width, { min, max });
    const stretched = elongatedWidth(line);
    const gaps = line.length - 1;
    return {
      words: line,
      spacing: stretch && gaps > 0 ? Math.max(0, (width - stretched) / gaps) : 0,
    };
  });
}

const fontStack = () => (state.family ? `"${state.family}", serif` : "serif");

function syncFont() {
  const size = document.getElementById("size").valueAsNumber;
  const width = document.getElementById("width").valueAsNumber;
  const leading = document.getElementById("leading").value;
  const font = `${size}px ${fontStack()}`;
  ctx.font = font;
  document.getElementById("out").style.font = font;
  document.getElementById("out").style.lineHeight = leading;
  document.getElementById("out").style.width = `${width}px`;
  document.getElementById("size-out").textContent = TEXT.px(num(size));
  document.getElementById("width-out").textContent = TEXT.px(num(width));
  document.getElementById("leading-out").textContent = num(leading);
  document.getElementById("min-out").textContent = num(document.getElementById("min").value);
  document.getElementById("max-out").textContent = num(document.getElementById("max").value);
}

// The pattern textarea wins over the selection when it holds anything.
function syncPatternSet() {
  const text = document.getElementById("patterns").value.trim();
  const status = document.getElementById("pattern-status");
  const previous = state.set;
  try {
    state.set = text ? compilePatternText(text) : builtinPatternSet(document.getElementById("set").value);
    state.syriac = !text && document.getElementById("set").value.startsWith("syriac");
    status.textContent = text ? TEXT.compiled : "";
    status.className = "status ok";
  } catch (error) {
    status.textContent = error.message || String(error);
    status.className = "status error";
    return;
  }
  if (previous) previous.free();
  document.getElementById("set").disabled = text !== "";
}

// The same splice as `elongated`, with each run wrapped so it can be measured
// for its priority label and skipped when an edit is read back.
function wordHtml(word) {
  let out = "";
  let end = word.text.length;
  for (const point of [...word.points].reverse()) {
    if (point.count) {
      out =
        `<span class="kashida" data-priority="${point.priority}">${TATWEEL.repeat(point.count)}</span>` +
        esc(word.text.slice(point.offset, end)) +
        out;
      end = point.offset;
    }
  }
  return esc(word.text.slice(0, end)) + out;
}

// Draws each kashida priority number into the #labels overlay.
// When `show` is false the overlay is emptied.
function paintLabels(show) {
  const labels = document.getElementById("labels");
  const runs = show ? document.getElementById("out").querySelectorAll(".kashida") : [];
  const origin = labels.getBoundingClientRect();
  const ascent = ctx.measureText(TATWEEL).fontBoundingBoxAscent;
  const lift = 0.6 * document.getElementById("size").valueAsNumber;
  labels.innerHTML = Array.from(runs, (span) => {
    const box = span.getBoundingClientRect();
    const x = box.left + box.width / 2 - origin.left;
    const y = box.top - origin.top + ascent - lift;
    return `<span style="left: ${x.toFixed(1)}px; top: ${y.toFixed(1)}px">${num(span.dataset.priority)}</span>`;
  }).join("");
}

function paint(lines) {
  document.getElementById("out").innerHTML = lines
    .map((line) => {
      const style = line.spacing ? ` style="word-spacing: ${line.spacing.toFixed(3)}px"` : "";
      return `<div class="line"${style}>${line.words.map(wordHtml).join(" ")}</div>`;
    })
    .join("");
}

function lineNodes() {
  return [...document.getElementById("out").childNodes].filter(
    (node) => node.nodeType !== Node.TEXT_NODE || node.data.trim() !== "",
  );
}

function textNodes(root) {
  if (root.nodeType === Node.TEXT_NODE) return [root];
  if (root.classList?.contains("kashida")) return [];
  return [...root.childNodes].flatMap(textNodes);
}

function readParagraph() {
  const selection = getSelection();
  const range = selection.rangeCount ? selection.getRangeAt(0) : null;
  const entries = [];
  let text = "";
  let caret = null;
  lineNodes().forEach((line, index) => {
    if (index) text += " ";
    for (const node of textNodes(line)) {
      if (node === range?.startContainer) caret = text.length + range.startOffset;
      entries.push({ node, start: text.length });
      text += node.data;
    }
  });
  return { text, caret, entries };
}

function setCaret(offset) {
  if (offset === null) return;
  const { entries } = readParagraph();
  const target =
    entries.find(({ node, start }) => offset <= start + node.data.length) ?? entries.at(-1);
  if (!target) return;
  const position = Math.min(Math.max(0, offset - target.start), target.node.data.length);
  getSelection().collapse(target.node, position);
}

function renderProbe() {
  const word = document.getElementById("probe").value.trim();
  const target = document.getElementById("probe-out");
  if (!word || !state.set) {
    target.innerHTML = "";
    return;
  }
  const [cleaned, points] = findPoints(word);
  if (!points.length) {
    target.innerHTML = `<p>${TEXT.none}</p>`;
    return;
  }
  const clusters = clustersOf(cleaned);
  const rows = points.map((point) => {
    const preview =
      clusters.slice(0, point.index + 1).join("") +
      TATWEEL.repeat(3) +
      clusters.slice(point.index + 1).join("");
    return `<tr><td>${num(point.priority)}</td><td>${num(point.index)}</td>
      <td class="sample" dir="rtl" style='font-family: ${fontStack()}'>${esc(preview)}</td></tr>`;
  });
  const head = TEXT.head.map((name) => `<th>${name}</th>`).join("");
  target.innerHTML = `<table><thead><tr>${head}
    </tr></thead><tbody>${rows.join("")}</tbody></table>`;
}

function render() {
  syncFont();
  if (!state.set) return;

  const justified = document.getElementById("justify").checked;
  const kashida = document.getElementById("kashida").checked;
  const highlight = document.getElementById("highlight").checked;
  document.getElementById("kashida").disabled = !justified;
  document.getElementById("highlight").disabled = !justified || !kashida;
  document.getElementById("out").classList.toggle("highlight", highlight);

  const width = document.getElementById("width").valueAsNumber;
  const lines = layout({
    text: state.text,
    width,
    min: document.getElementById("min").valueAsNumber,
    max: document.getElementById("max").valueAsNumber,
    justified,
    kashida,
    syriac: state.syriac,
  });
  paint(lines);
  paintLabels(highlight);

  const kashidas = lines
    .flatMap((line) => line.words)
    .flatMap((word) => word.points)
    .reduce((sum, point) => sum + point.count, 0);
  document.getElementById("stat").textContent = justified
    ? TEXT.stat(lines.length, kashidas)
    : TEXT.lines(lines.length);

  renderProbe();
}

const loadFaces = () => document.fonts.load(`1em ${fontStack()}`, state.text);

async function repaint() {
  render();
  await loadFaces();
  render();
}

const googleFontLink = document.head.appendChild(document.createElement("link"));
googleFontLink.rel = "stylesheet";

const chooseFont = document.getElementById("font-file-name").textContent;

async function useGoogleFont(family) {
  document.getElementById("font-file-name").textContent = chooseFont;
  document.getElementById("google-font").value = family;
  state.family = family;
  render();
  googleFontLink.href = `https://fonts.googleapis.com/css2?family=${encodeURIComponent(family)}&display=swap`;
  await new Promise((done) => (googleFontLink.onload = googleFontLink.onerror = done));
  await repaint();
}

async function useSample(script) {
  const sets = document.getElementById("set");
  state.text = SAMPLES[script];
  sets.value = [...sets.options]
    .map((option) => option.value)
    .find((name) => name.startsWith(script.toLowerCase())) ?? sets.value;
  syncPatternSet();
  if (document.getElementById("google-font").selectedIndex < 0) return repaint();
  await useGoogleFont(GOOGLE_FONTS[script][0]);
}

async function loadFont(file) {
  if (!file) return;
  const family = `loaded-${++state.faces}`;
  const face = new FontFace(family, await file.arrayBuffer());
  try {
    await face.load();
  } catch {
    document.getElementById("font-file-name").textContent = TEXT.failed(file.name);
    return;
  }
  document.fonts.add(face);
  document.getElementById("google-font").selectedIndex = -1;
  state.family = family;
  document.getElementById("font-file-name").textContent = file.name;
  render();
}

async function main() {
  await init();

  document.getElementById("version").textContent = TEXT.version(version());

  document.getElementById("set").innerHTML = builtinPatternSetNames().map((name) => `<option>${name}</option>`).join("");

  const sample = document.getElementById("sample");
  const fonts = document.getElementById("google-font");
  fonts.innerHTML = [...sample.options]
    .map((script) => {
      const families = GOOGLE_FONTS[script.value].map((family) => `<option>${family}</option>`);
      return `<optgroup label="${script.textContent}">${families.join("")}</optgroup>`;
    })
    .join("");

  for (const id of ["justify", "kashida", "highlight", "keep", "size", "width", "leading", "min", "max"]) {
    document.getElementById(id).addEventListener("input", render);
  }
  document.getElementById("out").addEventListener("input", () => {
    const { text, caret } = readParagraph();
    state.text = text;
    render();
    setCaret(caret);
  });
  sample.addEventListener("change", (event) => useSample(event.target.value));
  document.getElementById("probe").addEventListener("input", renderProbe);
  const recompile = () => {
    syncPatternSet();
    render();
  };
  document.getElementById("set").addEventListener("change", recompile);
  let pending;
  document.getElementById("patterns").addEventListener("input", () => {
    clearTimeout(pending);
    pending = setTimeout(recompile, 250);
  });
  document.getElementById("font-file-name").addEventListener("click", () => document.getElementById("font-file").click());
  document.getElementById("font-file").addEventListener("change", (event) => loadFont(event.target.files[0]));
  fonts.addEventListener("change", (event) => useGoogleFont(event.target.value));

  return useSample(sample.value);
}

main();
