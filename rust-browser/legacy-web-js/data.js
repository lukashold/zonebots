// Zonebots - static game data: units, themes, levels, texts
(function () {
  const ZG = globalThis.ZG = globalThis.ZG || {};

  ZG.TILE = 32;
  ZG.T = { GROUND: 0, ROAD: 1, ROCK: 2, WATER: 3, ROUGH: 4, BRIDGE: 5, BOULDER: 6 };

  // deterministic PRNG (mulberry32)
  ZG.rng = function (seed) {
    let s = seed >>> 0;
    return function () {
      s = (s + 0x6D2B79F5) >>> 0;
      let t = s;
      t = Math.imul(t ^ (t >>> 15), t | 1);
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  };
  ZG.clamp = (v, a, b) => v < a ? a : v > b ? b : v;

  // cls: robot | vehicle | gun.  n = squad size, rl = reload (s), sp = speed (px/s)
  ZG.UNITS = {
    grunt:    { cls: 'robot',   tech: 1, time: 22,  n: 4, hp: 40,  sp: 58,  range: 115, dmg: 6,  rl: 0.55, w: 'bullet',  r: 7,  armor: 0 },
    psycho:   { cls: 'robot',   tech: 1, time: 30,  n: 3, hp: 45,  sp: 58,  range: 125, dmg: 4,  rl: 0.16, w: 'bullet',  r: 7,  armor: 0 },
    tough:    { cls: 'robot',   tech: 2, time: 42,  n: 3, hp: 85,  sp: 48,  range: 155, dmg: 34, rl: 1.9,  w: 'rocket',  r: 8,  armor: 0, splash: 26 },
    sniper:   { cls: 'robot',   tech: 2, time: 40,  n: 2, hp: 40,  sp: 58,  range: 240, dmg: 14, rl: 1.7,  w: 'bullet',  r: 7,  armor: 0 },
    pyro:     { cls: 'robot',   tech: 3, time: 50,  n: 3, hp: 70,  sp: 54,  range: 72,  dmg: 7,  rl: 0.1,  w: 'flame',   r: 7,  armor: 0 },
    laser:    { cls: 'robot',   tech: 4, time: 62,  n: 3, hp: 85,  sp: 58,  range: 190, dmg: 18, rl: 0.55, w: 'laser',   r: 7,  armor: 0 },
    jeep:     { cls: 'vehicle', tech: 1, time: 32,  n: 1, hp: 110, sp: 105, range: 125, dmg: 3,  rl: 0.35, w: 'bullet',  r: 12, armor: 0.3 },
    // found on the original maps only, never built: unarmed
    truck:    { cls: 'vehicle', tech: 9, time: 99,  n: 1, hp: 160, sp: 90,  range: 0,   dmg: 0,  rl: 1,    w: null,      r: 14, armor: 0.4 },
    firetruck: { cls: 'vehicle', tech: 9, time: 99, n: 1, hp: 160, sp: 90,  range: 0,   dmg: 0,  rl: 1,    w: null,      r: 14, armor: 0.4 },
    light:    { cls: 'vehicle', tech: 2, time: 50,  n: 1, hp: 260, sp: 88,  range: 165, dmg: 32, rl: 1.5,  w: 'shell',   r: 14, armor: 0.6, splash: 22 },
    medium:   { cls: 'vehicle', tech: 3, time: 72,  n: 1, hp: 420, sp: 68,  range: 185, dmg: 48, rl: 1.7,  w: 'shell',   r: 15, armor: 0.7, splash: 26 },
    heavy:    { cls: 'vehicle', tech: 4, time: 100, n: 1, hp: 680, sp: 48,  range: 205, dmg: 75, rl: 2.0,  w: 'shell',   r: 17, armor: 0.8, splash: 30 },
    launcher: { cls: 'vehicle', tech: 4, time: 90,  n: 1, hp: 200, sp: 58,  range: 310, dmg: 85, rl: 3.6,  w: 'missile', r: 14, armor: 0.4, splash: 38, minRange: 70 },
    apc:      { cls: 'vehicle', tech: 2, time: 45,  n: 1, hp: 300, sp: 82,  range: 120, dmg: 3,  rl: 0.35, w: 'bullet',      r: 15, armor: 0.6, cargo: 6 },
    crane:    { cls: 'vehicle', tech: 1, time: 40,  n: 1, hp: 200, sp: 70,  range: 0,   dmg: 0,  rl: 1,    w: null,      r: 14, armor: 0.5, crane: true },
    gatling:  { cls: 'gun',     tech: 1, time: 35,  n: 1, hp: 260, sp: 0,   range: 175, dmg: 5,  rl: 0.14, w: 'bullet',  r: 13, armor: 0.5 },
    cannon:   { cls: 'gun',     tech: 2, time: 55,  n: 1, hp: 380, sp: 0,   range: 215, dmg: 42, rl: 1.6,  w: 'shell',   r: 14, armor: 0.6, splash: 24 },
    howitzer: { cls: 'gun',     tech: 3, time: 70,  n: 1, hp: 340, sp: 0,   range: 360, dmg: 60, rl: 3.0,  w: 'shell',   r: 14, armor: 0.6, splash: 34, minRange: 100 },
    mturret:  { cls: 'gun',     tech: 4, time: 80,  n: 1, hp: 320, sp: 0,   range: 300, dmg: 80, rl: 3.2,  w: 'missile', r: 14, armor: 0.5, splash: 36 }
  };

  ZG.THEMES = [
    { key: 'desert',  g: ['#c9a15c', '#c39a55', '#cfa763'], speck: '#a9843f', rough: '#ab8848', roughHi: '#8e6f36',
      road: '#9f8a66', roadEdge: '#85704e', rock: '#7d5c3c', rockHi: '#a98458', rockLo: '#4f3722',
      water: '#3b8fb0', waterHi: '#7cc3d8', shore: '#e2cf9a' },
    { key: 'volcano', g: ['#5b4b45', '#554641', '#614f48'], speck: '#3f322e', rough: '#473a35', roughHi: '#2f2522',
      road: '#7a6c62', roadEdge: '#5b4f47', rock: '#2d2523', rockHi: '#52433e', rockLo: '#161110',
      water: '#e25a1c', waterHi: '#ffc04a', shore: '#7a2a10' },
    { key: 'arctic',  g: ['#e3edf1', '#dbe7ec', '#ecf4f7'], speck: '#c3d3da', rough: '#c6d9e1', roughHi: '#a4bcc7',
      road: '#a9b6bc', roadEdge: '#8b999f', rock: '#8499a6', rockHi: '#c9d9e1', rockLo: '#566a76',
      water: '#2f6f9c', waterHi: '#8fc1de', shore: '#f7fbfd' },
    { key: 'jungle',  g: ['#4f8a3c', '#488237', '#569242'], speck: '#3a6c2b', rough: '#35692a', roughHi: '#22501b',
      road: '#927d50', roadEdge: '#75633c', rock: '#5b6451', rockHi: '#87937a', rockLo: '#363d2f',
      water: '#2f7f86', waterHi: '#74c1c2', shore: '#86a85a' },
    { key: 'city',    g: ['#8b8e91', '#85888b', '#919497'], speck: '#74777a', rough: '#75787b', roughHi: '#5d6063',
      road: '#5c5f63', roadEdge: '#d6c873', rock: '#4a4e57', rockHi: '#737885', rockLo: '#2b2e34',
      water: '#35607c', waterHi: '#6f9db8', shore: '#a9acaf' }
  ];

  // 20 original levels in 5 worlds. Layouts are generated deterministically from these parameters.
  const sizes = [[3, 2], [3, 2], [3, 3], [4, 3], [4, 3], [4, 3], [5, 3], [4, 4], [5, 3], [5, 4],
                 [5, 3], [5, 4], [6, 3], [5, 4], [6, 4], [6, 4], [6, 3], [6, 4], [7, 4], [7, 4]];
  const rivers = [0, 0, 1, 0, 1, 2, 0, 3, 0, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 3];
  // i = strength 0..19 (tech, AI skill, starting forces); o may override theme, seed and river
  ZG.makeLevel = function (i, cols, rows, o) {
    o = o || {};
    const theme = o.theme != null ? o.theme : Math.floor(i / 4);
    return {
      idx: i, theme: theme, cols: cols, rows: rows, custom: !!o.custom,
      sw: 12 + (i % 2), sh: 10 + ((i >> 1) % 2),
      seed: o.seed != null ? o.seed : 1000 + i * 7919,
      rock: 0.74 - 0.005 * i - (theme === 1 ? 0.03 : 0),
      rough: 0.7,
      river: o.river != null ? o.river : rivers[i],
      crates: i < 2 ? 0 : 1 + (i % 2),
      maxTech: i < 2 ? 1 : i < 5 ? 2 : i < 10 ? 3 : 4,
      fortTech: Math.min(4, 1 + Math.floor(i / 6)),
      neutral: i < 1 ? 0 : 1 + (i % 3),
      guns: Math.floor(i / 3),
      start: [5, 4 + Math.floor(i / 4)],
      aiRate: 0.55 + 0.03 * i,
      aiInt: 6 - 0.22 * i,
      aiRush: 16 - Math.floor(i / 3),
      aiBatch: 1 + Math.floor(i / 6),
      aiWait: Math.max(5, 45 - 2 * i),
      aiMin: 6 + Math.floor(i / 2)
    };
  };
  ZG.LEVELS = sizes.map((sz, i) => ZG.makeLevel(i, sz[0], sz[1]));
  // difficulty: 0 easy, 1 normal, 2 hard
  ZG.DIFF = [
    { rate: 0.5, int: 1.8, wait: 1.8, bonus: 4, rush: 1.3, batch: 0 },
    { rate: 1, int: 1, wait: 1, bonus: 0, rush: 1, batch: 0 },
    { rate: 1.5, int: 0.55, wait: 0.5, bonus: 0, rush: 0.85, batch: 1 },
    { rate: 2, int: 0.35, wait: 0.22, bonus: 0, rush: 0.65, batch: 2 }
  ];

  ZG.TEXT = {
    de: {
      sub: 'Echtzeit-Strategie um Sektoren, Flaggen und Fabriken',
      worlds: ['Staubbecken', 'Glutfelder', 'Frostschelf', 'Grünes Labyrinth', 'Eisenstadt'],
      level: 'Level', locked: 'Gesperrt', help: 'Anleitung', back: 'Zurück', lang: 'English',
      helpText: [
        '<b>Ziel:</b> Schicke einen Roboter durch das Tor des gegnerischen Forts, zerstöre das Fort oder vernichte alle Roboter und bemannten Fahrzeuge des Gegners. Umgekehrt gilt dasselbe für dich.',
        '<b>Sektoren:</b> Berührt eine Einheit die Flagge, gehört dir der Sektor samt Fabrik oder Werkstatt. Je mehr Sektoren du hältst, desto schneller bauen deine Fabriken. Beschädigte Fabriken bauen langsamer.',
        '<b>Trupps:</b> Roboter kommen als Trupp aus der Fabrik. Ein Klick wählt den ganzen Trupp, ein zweiter Klick auf ein Mitglied nur diesen Roboter, Umschalt+Klick nimmt einzelne dazu oder weg. Schickst du nur einen Teil los, wird er zu einem eigenen Trupp.',
        '<b>Besatzung:</b> Fahrzeuge und Geschütze brauchen einen Roboter. Leere (graue) Fahrzeuge und Geschütze bemannst du, indem du einen Roboter hinschickst. Scharfschützen können Besatzungen ausschalten.',
        '<b>Transporter:</b> Roboter auf den eigenen Transporter schicken – bis zu 6 steigen ein. Mit U oder dem Knopf „Aussteigen“ wieder absetzen.',
        '<b>Kran:</b> Repariert beschädigte eigene Gebäude und baut zerstörte Brücken wieder auf – Kran auswählen und auf das Gebäude oder die Brückenstelle klicken.',
        '<b>Brücken und Felsbrocken:</b> Lassen sich mit Granaten, Raketen und Granaten-Geschossen zerstören. Angriffsbefehl (F) auf die Brücke bzw. den Felsen.',
        '<b>Granatenkisten:</b> Roboter, die eine Kiste einsammeln, werfen Granaten auf Fahrzeuge, Geschütze und Gebäude.',
        '<b>Steuerung klassisch (wie Z):</b> Linksklick auf eigene Einheit = auswählen · Linksklick woandershin = Befehl · Rechtsklick = Auswahl aufheben · Rahmen ziehen = mehrere auswählen.',
        '<b>Steuerung modern:</b> Linksklick / Rahmen = auswählen · Rechtsklick = Befehl.',
        '<b>Tasten:</b> F = Angriffsbefehl · X = Stopp · U = Aussteigen · Strg+1–9 / 1–9 = Gruppen · Strg+A = alle · Pfeile/WASD = scrollen · Mausrad = Zoom · Leertaste = zum Fort · P = Pause. Touch: Tippen = auswählen/Befehl, Ziehen = scrollen, zwei Finger = Zoom.'
      ],
      units: { grunt: 'Rekrut', psycho: 'Schnellfeuer', tough: 'Raketenschütze', sniper: 'Scharfschütze', pyro: 'Flammenwerfer', laser: 'Laserschütze',
        jeep: 'Jeep', light: 'Leichter Panzer', medium: 'Mittlerer Panzer', heavy: 'Schwerer Panzer', launcher: 'Raketenwerfer',
        gatling: 'MG-Geschütz', cannon: 'Kanone', mturret: 'Raketenturm', apc: 'Transporter', crane: 'Kran', howitzer: 'Haubitze', truck: 'Lastwagen', firetruck: 'Löschfahrzeug' },
      b: { fort: 'Fort', robot: 'Roboterfabrik', vehicle: 'Fahrzeugfabrik', repair: 'Werkstatt', radar: 'Radarstation' },
      builds: 'baut', tech: 'Technik', sectors: 'Sektoren', selected: 'ausgewählt',
      captured: 'Sektor erobert', lost: 'Sektor verloren', ready: 'fertig', alert: 'Dein Fort wird angegriffen!',
      crewed: 'Fahrzeug bemannt', driver: 'Fahrer ausgeschaltet',
      win: 'Sieg!', lose: 'Niederlage', next: 'Nächstes Level', retry: 'Nochmal', menu: 'Menü',
      paused: 'Pause', resume: 'Weiter', time: 'Zeit', kills: 'Zerstört', losses: 'Verluste', allDone: 'Alle 20 Level geschafft!',
      repairHint: 'Repariert eigene Einheiten in der Nähe.',
      diff: ['Leicht', 'Normal', 'Schwer', 'Brutal'], diffLabel: 'Schwierigkeit', skirmish: 'Freies Gefecht', campaign: 'Kampagne',
      size: 'Größe', sizes: ['Klein', 'Mittel', 'Groß', 'Riesig'], world: 'Welt', strength: 'Technik', startSk: 'Gefecht starten',
      crate: 'Granaten aufgenommen', best: 'Bestzeit', newBest: 'Neue Bestzeit!', left: 'noch', rally: 'Sammelpunkt gesetzt',
      selAll: 'Alle', selNone: 'Abwählen', stop: 'Stopp', amoveBtn: 'Angriff', seedLabel: 'Karten-Nr.',
      unload: 'Aussteigen', ctrlTitle: 'Steuerung', ctrl: ['Klassisch (wie Z)', 'Modern'], squad: 'Trupp', empty: 'unbemannt', aboard: 'an Bord',
      bridgeDown: 'Brücke zerstört', bridgeUp: 'Brücke wieder aufgebaut', ourBldDead: 'Eigenes Gebäude zerstört', bldDead: 'Gegnerisches Gebäude zerstört',
      stormed: 'Fort erstürmt!', boarded: 'Eingestiegen', unloaded: 'Ausgestiegen', fixed: 'Reparatur fertig', rockGone: 'Felsbrocken gesprengt',
      craneHint: 'Gebäude oder zerstörte Brücke anklicken', apcHint: 'Roboter hierher schicken zum Einsteigen', gunHint: 'Feste Stellung', destroyed: 'zerstört', neutral: 'neutral',
      enemy: 'Gegner', choose: 'Bauauftrag wählen', cancel: 'Anhalten',
      zTitle: 'Original-Z (optional)', zPick: 'Z-Ordner wählen…', zForget: 'Entfernen', zLoading: 'Lese Dateien…',
      zHint: 'Besitzt du das Originalspiel Z (z. B. von GOG)? Wähle seinen Installationsordner oder eine ZIP-Datei davon (praktisch für den Mac), dann spielst du die 20 Originalkarten mit Originalgrafik, Musik und Stimmen. Die Dateien werden nur in deinem Browser gelesen und gespeichert, nichts wird hochgeladen.',
      zNone: 'In diesem Ordner wurden keine Z-Karten gefunden.', zLoaded: 'Originalkarten geladen:',
      mPlay: 'Neues Spiel', mOrig: 'Originalkarten', mSkirm: 'Freies Gefecht', mOpts: 'Optionen', langTitle: 'Sprache',
      fullHint: 'Vollbild: F11 (Mac: Ctrl+Cmd+F) oder ⛶ im Spiel', zZip: 'ZIP-Datei wählen…'
    },
    en: {
      sub: 'Real-time strategy about sectors, flags and factories',
      worlds: ['Dust Basin', 'Ember Fields', 'Frost Shelf', 'Green Maze', 'Iron City'],
      level: 'Level', locked: 'Locked', help: 'How to play', back: 'Back', lang: 'Deutsch',
      helpText: [
        '<b>Goal:</b> Send a robot through the enemy fort\'s gate, destroy the fort, or wipe out all enemy robots and crewed vehicles. The same goes for you.',
        '<b>Sectors:</b> Touch a flag with any unit to own the sector with its factory or workshop. The more sectors you hold, the faster your factories build. Damaged factories build slower.',
        '<b>Squads:</b> Robots leave the factory as a squad. One click selects the whole squad, a second click on a member selects just that robot, Shift+click adds or removes single units. Sending only part of a squad makes it a squad of its own.',
        '<b>Crews:</b> Vehicles and guns need a robot. Crew an empty (grey) vehicle or gun by sending a robot to it. Snipers can take out crews.',
        '<b>Personnel carrier:</b> Send robots onto your carrier – up to 6 climb in. Press U or the "Unload" button to drop them off.',
        '<b>Crane:</b> Repairs your damaged buildings and rebuilds destroyed bridges – select the crane and click the building or the bridge site.',
        '<b>Bridges and boulders:</b> Can be destroyed with grenades, rockets and shells. Use the attack order (F) on the bridge or boulder.',
        '<b>Grenade crates:</b> Robots that collect a crate throw grenades at vehicles, guns and buildings.',
        '<b>Classic controls (like Z):</b> Left click own unit = select · left click elsewhere = order · right click = deselect · drag = select several.',
        '<b>Modern controls:</b> Left click / drag = select · right click = order.',
        '<b>Keys:</b> F = attack order · X = stop · U = unload · Ctrl+1–9 / 1–9 = groups · Ctrl+A = all · Arrows/WASD = scroll · Wheel = zoom · Space = jump to fort · P = pause. Touch: tap = select/order, drag = scroll, two fingers = zoom.'
      ],
      units: { grunt: 'Grunt', psycho: 'Rapid Gunner', tough: 'Rocketeer', sniper: 'Marksman', pyro: 'Flamer', laser: 'Laser Trooper',
        jeep: 'Jeep', light: 'Light Tank', medium: 'Medium Tank', heavy: 'Heavy Tank', launcher: 'Missile Truck',
        gatling: 'MG Turret', cannon: 'Cannon', mturret: 'Missile Turret', apc: 'Personnel Carrier', crane: 'Crane', howitzer: 'Howitzer', truck: 'Truck', firetruck: 'Fire Truck' },
      b: { fort: 'Fort', robot: 'Robot Factory', vehicle: 'Vehicle Factory', repair: 'Workshop', radar: 'Radar Station' },
      builds: 'builds', tech: 'Tech', sectors: 'Sectors', selected: 'selected',
      captured: 'Sector captured', lost: 'Sector lost', ready: 'ready', alert: 'Your fort is under attack!',
      crewed: 'Vehicle crewed', driver: 'Driver taken out',
      win: 'Victory!', lose: 'Defeat', next: 'Next level', retry: 'Retry', menu: 'Menu',
      paused: 'Paused', resume: 'Resume', time: 'Time', kills: 'Destroyed', losses: 'Losses', allDone: 'All 20 levels completed!',
      repairHint: 'Repairs your units nearby.',
      diff: ['Easy', 'Normal', 'Hard', 'Brutal'], diffLabel: 'Difficulty', skirmish: 'Skirmish', campaign: 'Campaign',
      size: 'Size', sizes: ['Small', 'Medium', 'Large', 'Huge'], world: 'World', strength: 'Tech', startSk: 'Start skirmish',
      crate: 'Grenades picked up', best: 'Best time', newBest: 'New best time!', left: 'left', rally: 'Rally point set',
      selAll: 'All', selNone: 'Deselect', stop: 'Stop', amoveBtn: 'Attack', seedLabel: 'Map no.',
      unload: 'Unload', ctrlTitle: 'Controls', ctrl: ['Classic (like Z)', 'Modern'], squad: 'Squad', empty: 'no crew', aboard: 'aboard',
      bridgeDown: 'Bridge destroyed', bridgeUp: 'Bridge rebuilt', ourBldDead: 'Our building destroyed', bldDead: 'Enemy building destroyed',
      stormed: 'Fort stormed!', boarded: 'Boarded', unloaded: 'Unloaded', fixed: 'Repair complete', rockGone: 'Boulder blasted',
      craneHint: 'Click a building or a destroyed bridge', apcHint: 'Send robots here to board', gunHint: 'Fixed emplacement', destroyed: 'destroyed', neutral: 'neutral',
      enemy: 'Enemy', choose: 'Choose production', cancel: 'Halt',
      zTitle: 'Original Z (optional)', zPick: 'Choose Z folder…', zForget: 'Remove', zLoading: 'Reading files…',
      zHint: 'Do you own the original game Z (e.g. from GOG)? Pick its installation folder or a zip of it (handy on a Mac) to play the 20 original maps with original graphics, music and voices. The files are only read and stored in your browser, nothing is uploaded.',
      zNone: 'No Z maps found in that folder.', zLoaded: 'Original maps loaded:',
      mPlay: 'New Game', mOrig: 'Original Maps', mSkirm: 'Skirmish', mOpts: 'Options', langTitle: 'Language',
      fullHint: 'Full screen: F11 (Mac: Ctrl+Cmd+F) or ⛶ in the game', zZip: 'Choose zip file…'
    }
  };
})();
