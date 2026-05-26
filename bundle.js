#!/usr/bin/env node
/**
 * bundle_sw.js — Bundle all build assets into index.html with a fixed service-worker deployment.
 *
 * Usage:  node bundle_sw.js <source-folder> <deploy-folder> <base-path>
 */

const fs   = require('fs');
const path = require('path');
const zlib = require('zlib');

// ── CLI args ─────────────────────────────────────────────────────────────────
const srcFolder    = process.argv[2];
const deployFolder = process.argv[3];
const basePath     = (process.argv[4] || '').replace(/^\/|\/$/g, '');

if (!srcFolder || !deployFolder) {
    console.error('Usage: node bundle_sw.js <source-folder> <deploy-folder> [base-path]');
    process.exit(1);
}

if (!fs.existsSync(srcFolder)) {
    console.error(`Error: source folder not found: ${srcFolder}`);
    process.exit(1);
}

// ── Mime type map ────────────────────────────────────────────────────────────
const MIME_BY_EXT = {
    '.html':        'text/html',
    '.css':         'text/css',
    '.js':          'application/javascript',
    '.json':        'application/json',
    '.map':         'application/json',
    '.webmanifest': 'application/manifest+json',
    '.png':         'image/png',
    '.ico':         'image/x-icon',
    '.svg':         'image/svg+xml',
    '.wasm':        'application/wasm',
    '.woff':        'font/woff',
    '.woff2':       'font/woff2',
};

// ── Collect files recursively ────────────────────────────────────────────────
function walkDir(dir, base) {
    let results = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        const rel  = path.join(base, entry.name);
        if (entry.isDirectory()) {
            results = results.concat(walkDir(full, rel));
        } else if (entry.isFile()) {
            results.push(rel);
        }
    }
    return results;
}

const SKIP = new Set(['sw.js', 'index.html']);
const allFiles = walkDir(srcFolder, '').filter(f => !SKIP.has(f));

// ── Build the compressed ASSETS and MIME objects ─────────────────────────────
const compactAssets = {};
const mimeTypes = {};
let totalRaw = 0;
let totalCompact = 0;

for (const relPath of allFiles) {
    const absPath  = path.join(srcFolder, relPath);
    const buf      = fs.readFileSync(absPath);
    const ext      = path.extname(relPath).toLowerCase();
    const mime     = MIME_BY_EXT[ext] || 'application/octet-stream';
    const key      = relPath.split(path.sep).join('/');

    mimeTypes[key] = mime;
    totalRaw      += buf.length;

    // Minden asset tömörítése maxos (level 9) gzip-eléssel
    const gzBuf = zlib.gzipSync(buf, { level: 9 });
    compactAssets[key] = gzBuf.toString('base64');
    totalCompact += gzBuf.length;
}

// ── Marker used for JSON extraction ──────────────────────────────────────────
const MARKER = '__EMBEDDED_ASSETS_START__';

// ── Generate HTML by inserting all compressed assets before </body> ──────────
function generateAppIndexHtml(htmlFile, dataSection) {
    const html = fs.readFileSync(path.join(srcFolder, htmlFile), 'utf8');
    const bodyClose = html.lastIndexOf('</body>');
    if (bodyClose === -1) {
        console.error('Error: </body> not found in ' + htmlFile);
        process.exit(1);
    }
    return html.slice(0, bodyClose) + dataSection + '\n' + html.slice(bodyClose);
}

// ── Output generálás ─────────────────────────────────────────────────────────
const outFolder = basePath ? path.join(deployFolder, basePath) : deployFolder;
fs.mkdirSync(outFolder, { recursive: true });

// EGYETLEN LÉPCSŐ: Összes asset becsomagolása egy JSON-be, és injektálása az index.html-be
const allAssetsJson = JSON.stringify({ assets: compactAssets, mime: mimeTypes });
const appDataSection = `<script type="application/json" id="${MARKER}">\n${allAssetsJson}\n</script>`;
const outputHtml = generateAppIndexHtml("index.html", appDataSection);

// Fájlok kiírása a deploy mappába
const swSrc = path.join(srcFolder, 'sw.js');
if (!fs.existsSync(swSrc)) {
    console.error(`Error: sw.js not found in ${srcFolder}`);
    process.exit(1);
}
fs.copyFileSync(swSrc, path.join(outFolder, 'sw.js'));
fs.writeFileSync(path.join(outFolder, 'index.html'), outputHtml, 'utf8');

// ── Riport / Summary ─────────────────────────────────────────────────────────
const swSize   = fs.statSync(path.join(outFolder, 'sw.js')).size;
const htmlSize = Buffer.byteLength(outputHtml, 'utf8');
console.log(`Bundled ${allFiles.length} files — single-stage json-in-html + dioxus SPA mode`);
console.log(`  Base path: ${basePath ? '/' + basePath + '/' : '/ (root)'}`);
console.log(`  Raw assets size: ${(totalRaw / 1024).toFixed(1)} KB`);
console.log(`  Output sw.js: ${(swSize / 1024).toFixed(1)} KB (copied)`);
console.log(`  Output index.html (with all embedded assets): ${(htmlSize / 1024).toFixed(1)} KB`);
console.log(`  Compressed assets size: ${(totalCompact / 1024).toFixed(1)} KB (${(100 * totalCompact / totalRaw).toFixed(0)}% of raw)`);
console.log(`  Deploy folder: ${outFolder}/`);
console.log('');
for (const relPath of allFiles) {
    const size = fs.statSync(path.join(srcFolder, relPath)).size;
    const mime = mimeTypes[relPath.split(path.sep).join('/')];
    console.log(`  ${relPath} (${(size / 1024).toFixed(1)} KB) → ${mime}`);
}
