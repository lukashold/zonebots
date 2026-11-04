// The hosted root and local server share one HTML entry, with relative assets
// resolved beneath independent/ only for the hosted root.
'use strict';
const fs=require('node:fs'),path=require('node:path'),root=path.resolve(__dirname,'..');
const html=fs.readFileSync(path.join(root,'dist/independent/index.html'),'utf8');
const hosted=html.replace(/\b(src|href)="(?!(?:[a-z][a-z0-9+.-]*:|\/|#))([^"]+)"/gi,(_,attr,url)=>`${attr}="independent/${url}"`);
fs.writeFileSync(path.join(root,'dist/index.html'),hosted);
