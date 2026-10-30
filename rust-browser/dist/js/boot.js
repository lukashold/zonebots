(async function () {
  const cover = document.createElement('div');
  cover.id = 'engineBoot';
  cover.style.cssText = 'position:fixed;inset:0;z-index:9999;background:#101317;color:#ffd23a;display:grid;place-content:center;text-align:center;font:24px monospace;padding:32px';
  cover.textContent = 'Z · Rust — Spiel wird geladen …';
  document.body.appendChild(cover);
  try {
    const response = await fetch('engine.wasm');
    if (!response.ok) throw new Error('engine.wasm: HTTP ' + response.status);
    await ZRust.load(await response.arrayBuffer());
    const main = document.createElement('script'); main.src = 'js/main.js';
    main.onload = () => cover.remove();
    main.onerror = () => { cover.textContent = 'Spieloberfläche konnte nicht geladen werden. Bitte neu laden.'; };
    document.body.appendChild(main);
  } catch (error) {
    cover.textContent = 'Start fehlgeschlagen: ' + error.message + '\nBitte Start-Spiel.cmd verwenden; die HTML-Datei benötigt einen lokalen Webserver.';
    console.error(error);
  }
})();
