// Zonebots - our own sound when no original Z files are loaded: synthesised effects, a small rock band
// for the music, and the browser's speech synthesis for the robots' voices. Nothing here is a recording.
(function () {
  const ZG = window.ZG, S = ZG.Synth = {};
  let ac = null, master = null, musicBus = null, effectsBus = null, noise = null, shaper = null;
  const settings = { music: 0.55, effects: 0.7, speech: 0.9, enabled: true };
  let ducked = false;
  function mix() {
    if (!ac) return;
    master.gain.setTargetAtTime(settings.enabled ? 0.8 : 0, ac.currentTime, 0.04);
    musicBus.gain.setTargetAtTime(settings.music * (ducked ? 0.1 : 0.4), ac.currentTime, 0.12);
    effectsBus.gain.setTargetAtTime(settings.effects, ac.currentTime, 0.04);
  }
  S.configure = function(values) { for(const k of ['music','effects','speech']) if(Number.isFinite(values[k])) settings[k]=Math.max(0,Math.min(1,values[k])); if('enabled' in values)settings.enabled=!!values.enabled; mix(); if((!settings.enabled || !settings.speech) && window.speechSynthesis) { speechSynthesis.cancel(); queue.length=0; speaking=false; ducked=false; mix(); } };
  S.unlock = () => { ctx(); if(window.speechSynthesis) speechSynthesis.getVoices(); };
  function ctx() {
    if (!ac) {
      ac = new (window.AudioContext || window.webkitAudioContext)();
      master = ac.createGain(); master.gain.value = 0.8; master.connect(ac.destination);
      musicBus = ac.createGain(); musicBus.gain.value = 0.28; musicBus.connect(master);
      effectsBus = ac.createGain(); effectsBus.connect(master); mix();
      const n = ac.sampleRate, b = ac.createBuffer(1, n, n), d = b.getChannelData(0);
      for (let i = 0; i < n; i++) d[i] = Math.random() * 2 - 1;
      noise = b;
      shaper = new Float32Array(1024);
      for (let i = 0; i < 1024; i++) { const x = i / 512 - 1; shaper[i] = Math.tanh(x * 6); }
    }
    if (ac.state === 'suspended') ac.resume();
    return ac;
  }
  S.ctx = ctx;

  // ---------- building blocks ----------
  function env(g, t, a, peak, d) { g.gain.setValueAtTime(0.0001, t); g.gain.exponentialRampToValueAtTime(peak, t + a); g.gain.exponentialRampToValueAtTime(0.0001, t + a + d); }
  function noiseHit(t, dest, f0, f1, type, peak, dur, q) {
    const s = ac.createBufferSource(), f = ac.createBiquadFilter(), g = ac.createGain();
    s.buffer = noise; s.loop = true; f.type = type; f.frequency.setValueAtTime(f0, t); f.frequency.exponentialRampToValueAtTime(f1, t + dur); f.Q.value = q || 0.7;
    env(g, t, 0.003, peak, dur); s.connect(f); f.connect(g); g.connect(dest);
    s.start(t, Math.random() * 0.5); s.stop(t + dur + 0.05);
  }
  function toneHit(t, dest, type, f0, f1, peak, dur) {
    const o = ac.createOscillator(), g = ac.createGain();
    o.type = type; o.frequency.setValueAtTime(f0, t); o.frequency.exponentialRampToValueAtTime(f1, t + dur);
    env(g, t, 0.004, peak, dur); o.connect(g); g.connect(dest); o.start(t); o.stop(t + dur + 0.05);
  }

  // ---------- sound effects ----------
  S.sfx = function (kind, vol) {
    const t = ctx().currentTime, v = vol == null ? 1 : vol;
    switch (kind) {
      case 'mg': for (let i = 0; i < 3; i++) { noiseHit(t + i * 0.055, effectsBus, 3200, 700, 'bandpass', 0.5 * v, 0.045, 1.2); toneHit(t + i * 0.055, effectsBus, 'square', 140, 60, 0.08 * v, 0.03); } break;
      case 'rifle': noiseHit(t, effectsBus, 5000, 400, 'highpass', 0.6 * v, 0.12); toneHit(t, effectsBus, 'triangle', 300, 70, 0.25 * v, 0.1); break;
      case 'cannon': toneHit(t, effectsBus, 'sine', 120, 35, 0.9 * v, 0.35); noiseHit(t, effectsBus, 1800, 120, 'lowpass', 0.7 * v, 0.4); break;
      case 'rocket': noiseHit(t, effectsBus, 600, 3500, 'bandpass', 0.35 * v, 0.45, 2); toneHit(t, effectsBus, 'sawtooth', 200, 520, 0.06 * v, 0.4); break;
      case 'flame': noiseHit(t, effectsBus, 900, 400, 'lowpass', 0.4 * v, 0.25); break;
      case 'laser': toneHit(t, effectsBus, 'sawtooth', 2200, 300, 0.12 * v, 0.16); toneHit(t, effectsBus, 'square', 1500, 200, 0.05 * v, 0.14); break;
      case 'boomS': noiseHit(t, effectsBus, 1400, 90, 'lowpass', 0.8 * v, 0.45); toneHit(t, effectsBus, 'sine', 90, 30, 0.6 * v, 0.35); break;
      case 'boomL': noiseHit(t, effectsBus, 900, 50, 'lowpass', 1.0 * v, 1.1); toneHit(t, effectsBus, 'sine', 70, 22, 0.9 * v, 0.9); noiseHit(t + 0.08, effectsBus, 3000, 200, 'bandpass', 0.3 * v, 0.6); break;
      case 'click': toneHit(t, effectsBus, 'square', 1200, 900, 0.06 * v, 0.03); break;
      case 'ready': toneHit(t, effectsBus, 'square', 660, 660, 0.08 * v, 0.08); toneHit(t + 0.09, effectsBus, 'square', 990, 990, 0.08 * v, 0.12); break;
      default: return false;
    }
    return true;
  };

  // ---------- music: drums, bass and a distorted guitar, scheduled ahead ----------
  const NOTE = n => 440 * Math.pow(2, (n - 69) / 12);
  // E minor riffs; one row per bar, numbers are MIDI notes (0 = rest)
  const SONGS = {
    battle: { bpm: 138, bars: [
      { root: 40, riff: [40, 0, 40, 40, 43, 0, 40, 45, 0, 40, 40, 47, 45, 0, 43, 0] },
      { root: 40, riff: [40, 0, 40, 40, 43, 0, 40, 45, 0, 40, 40, 50, 48, 0, 47, 0] },
      { root: 36, riff: [36, 0, 36, 36, 40, 0, 36, 43, 0, 36, 36, 43, 41, 0, 40, 0] },
      { root: 38, riff: [38, 0, 38, 38, 42, 0, 38, 45, 0, 43, 0, 42, 0, 40, 0, 38] }],
      kick: [1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0], snare: [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1] },
    battle2: { bpm: 126, bars: [
      { root: 45, riff: [45, 45, 0, 45, 48, 0, 50, 0, 45, 45, 0, 45, 52, 0, 50, 48] },
      { root: 43, riff: [43, 43, 0, 43, 47, 0, 48, 0, 43, 43, 0, 43, 50, 0, 48, 47] },
      { root: 41, riff: [41, 41, 0, 41, 45, 0, 47, 0, 41, 41, 0, 41, 48, 0, 47, 45] },
      { root: 40, riff: [40, 40, 0, 40, 43, 0, 45, 0, 47, 0, 45, 0, 43, 0, 40, 0] }],
      kick: [1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0], snare: [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0] },
    menu: { bpm: 96, bars: [
      { root: 40, riff: [40, 0, 0, 0, 47, 0, 0, 0, 45, 0, 0, 0, 43, 0, 0, 0] },
      { root: 36, riff: [36, 0, 0, 0, 43, 0, 0, 0, 40, 0, 0, 0, 43, 0, 0, 0] },
      { root: 38, riff: [38, 0, 0, 0, 45, 0, 0, 0, 42, 0, 0, 0, 45, 0, 0, 0] },
      { root: 35, riff: [35, 0, 0, 0, 42, 0, 0, 0, 39, 0, 0, 0, 42, 0, 0, 0] }],
      kick: [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], snare: [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0] }
  };
  function guitar(t, note, dur) {
    const f = NOTE(note + 12), mix = ac.createGain(), ws = ac.createWaveShaper(), lp = ac.createBiquadFilter(), g = ac.createGain();
    ws.curve = shaper; lp.type = 'lowpass'; lp.frequency.value = 2400;
    for (const m of [1, 1.5, 2]) { const o = ac.createOscillator(); o.type = 'sawtooth'; o.frequency.value = f * m; o.detune.value = (Math.random() - 0.5) * 12; o.connect(mix); o.start(t); o.stop(t + dur + 0.05); }
    mix.gain.value = 0.25; mix.connect(ws); ws.connect(lp); lp.connect(g); g.connect(musicBus);
    g.gain.setValueAtTime(0.0001, t); g.gain.exponentialRampToValueAtTime(0.32, t + 0.01); g.gain.setValueAtTime(0.32, t + dur * 0.7); g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
  }
  function bass(t, note, dur) {
    const o = ac.createOscillator(), lp = ac.createBiquadFilter(), g = ac.createGain();
    o.type = 'sawtooth'; o.frequency.value = NOTE(note - 12); lp.type = 'lowpass'; lp.frequency.value = 500;
    env(g, t, 0.01, 0.5, dur); o.connect(lp); lp.connect(g); g.connect(musicBus); o.start(t); o.stop(t + dur + 0.05);
  }
  function kick(t) { toneHit(t, musicBus, 'sine', 150, 40, 1.1, 0.18); }
  function snare(t) { noiseHit(t, musicBus, 3500, 1200, 'highpass', 0.55, 0.16); toneHit(t, musicBus, 'triangle', 220, 160, 0.25, 0.08); }
  function hat(t) { noiseHit(t, musicBus, 9000, 7000, 'highpass', 0.12, 0.03); }

  let song = null, songName = '', step = 0, nextT = 0, timer = 0;
  function schedule() {
    if (ac.state !== 'running') return;
    if (nextT < ac.currentTime - 0.2) nextT = ac.currentTime + 0.02;
    while (nextT < ac.currentTime + 0.15) {
      const sp = 60 / song.bpm / 4, bar = song.bars[Math.floor(step / 16) % song.bars.length], i = step % 16;
      if (song.kick[i]) kick(nextT);
      if (song.snare[i]) snare(nextT);
      if (i % 2 === 0) hat(nextT);
      const n = bar.riff[i];
      if (n) { guitar(nextT, n, sp * 0.9); if (songName !== 'menu' || i % 4 === 0) bass(nextT, n, sp * (songName === 'menu' ? 3.5 : 0.9)); }
      // Sixteen-bar arrangement: introduce an arpeggio and pads, then a drum fill.
      const section = Math.floor(step / 64) % 4;
      if (section > 0 && i % 4 === 2) toneHit(nextT, musicBus, 'triangle', NOTE(bar.root+24+[0,7,10,12][Math.floor(i/4)]), NOTE(bar.root+24+[0,7,10,12][Math.floor(i/4)]), 0.16, sp*1.8);
      if (i === 0) for(const interval of [0,7,12]) toneHit(nextT,musicBus,'sine',NOTE(bar.root+interval),NOTE(bar.root+interval),0.09,sp*14);
      if(section===3 && i>12) snare(nextT);
      nextT += sp; step++;
    }
  }
  S.music = function (name) {
    ctx(); // A repeated song request must still resume a suspended audio context.
    if (songName === name) return;
    S.stopMusic();
    if (name === 'win' || name === 'lose') {
      const t = ctx().currentTime + 0.05, notes = name === 'win' ? [52, 55, 59, 64, 64] : [52, 51, 50, 48, 40];
      notes.forEach((n, k) => { guitar(t + k * 0.22, n, k === notes.length - 1 ? 1.2 : 0.2); bass(t + k * 0.22, n, 0.2); });
      kick(t); snare(t + 0.88);
      return;
    }
    song = SONGS[name]; if (!song) return;
    ctx(); songName = name; step = 0; nextT = ac.currentTime + 0.05;
    timer = setInterval(schedule, 40); schedule();
  };
  S.stopMusic = function () { clearInterval(timer); timer = 0; songName = ''; };

  // ---------- voices: the robots' lines, spoken by the browser ----------
  const LINES = {
    ROB01: ['Yes sir!', 'Jawohl!'], ROB02: ['Yes sir!', 'Jawohl, Sir!'], ROB03: ['Yes sir!', 'Zu Befehl!'],
    ROB04: ['Unit reporting.', 'Einheit meldet sich.'], ROB05: ['Unit reporting.', 'Einheit bereit.'], ROB06: ['Unit reporting.', 'Einheit zur Stelle.'],
    ROB07: ['Grunts reporting.', 'Rekruten melden sich.'], ROB08: ['Psychos reporting.', 'Schnellfeuer meldet sich.'], ROB09: ['Snipers reporting.', 'Scharfschützen melden sich.'],
    ROB10: ['Toughs reporting.', 'Raketenschützen melden sich.'], ROB11: ['Lasers reporting.', 'Laserschützen melden sich.'], ROB12: ['Pyros reporting.', 'Flammenwerfer melden sich.'],
    ROB13: ["We're on our way.", 'Wir sind unterwegs.'], ROB14: ['Here we go!', 'Los geht\'s!'], ROB15: ['You got it.', 'Wird gemacht.'], ROB16: ['Moving in.', 'Rücken vor.'],
    ROB17: ['Okay.', 'Okay.'], ROB18: ['All right.', 'Alles klar.'], ROB19: ['No problem.', 'Kein Problem.'], ROB20: ['Over and out.', 'Over und Ende.'],
    ROB21: ['Affirmative.', 'Verstanden.'], ROB22: ['Going in.', 'Wir gehen rein.'], ROB23: ["Let's do it!", 'Packen wir\'s an!'], ROB24: ["Let's get 'em!", 'Holen wir sie uns!'],
    ROB25: ["We're under attack!", 'Wir werden angegriffen!'], ROB26: ["I said we're under attack!", 'Ich sagte, wir werden angegriffen!'], ROB27: ['Help! Help!', 'Hilfe! Hilfe!'],
    ROB28: ["They're all over us!", 'Die sind überall!'], ROB29: ["We're losing it!", 'Wir verlieren!'],
    ROB37: ['Target destroyed.', 'Ziel zerstört.'], ROB38: ['Bridge destroyed.', 'Brücke zerstört.'], ROB39: ['Building destroyed.', 'Gebäude zerstört.'],
    ROB40: ['Good hit!', 'Guter Treffer!'], ROB41: ['Nice one!', 'Sauber!'], ROB42: ['Oh yeah!', 'Oh ja!'], ROB43: ['Gotcha!', 'Erwischt!'], ROB44: ["Smokin'!", 'Heiß!'],
    ROB45: ['Cool!', 'Cool!'], ROB46: ['Wipe out!', 'Weggeputzt!'], ROB47: ['Bridge repaired.', 'Brücke repariert.'], ROB48: ['Building repaired.', 'Gebäude repariert.'],
    ROB49: ['Territory taken!', 'Gebiet erobert!'], ROB51: ['Gun captured!', 'Geschütz erobert!'], ROB52: ['Vehicle captured!', 'Fahrzeug erobert!'],
    ROB53: ['Grenades collected!', 'Granaten eingesammelt!'], ROB54: ["Let's take the fort!", 'Nehmen wir das Fort!'], ROB65: ["We've done it!", 'Geschafft!'],
    COMP01: ['Vehicle manufactured.', 'Fahrzeug hergestellt.'], COMP02: ['Robot manufactured.', 'Roboter hergestellt.'], COMP03: ['Gun manufactured.', 'Geschütz hergestellt.'],
    COMP04: ['Starting manufacture.', 'Produktion gestartet.'], COMP05: ['Manufacturing cancelled.', 'Produktion abgebrochen.'], COMP08: ['Territory lost.', 'Gebiet verloren.'],
    COMP09: ['Radar activated.', 'Radar aktiviert.'], COMP10: ["They're gonna win!", 'Die gewinnen gleich!'], COMP11: ["You've had it!", 'Du bist erledigt!'],
    COMP12: ["You're finished!", 'Aus und vorbei!'], COMP13: ["They're beating you!", 'Die schlagen dich!'], COMP15: ["You're gonna lose!", 'Du wirst verlieren!'],
    COMP16: ['Your fort is under attack!', 'Dein Fort wird angegriffen!'], COMP17: ["You're hopeless!", 'Hoffnungslos!'], COMP18: ['You are so hopeless!', 'Völlig hoffnungslos!']
  };
  let voices = null;
  function pickVoice(lang) {
    if (!('speechSynthesis' in window)) return null;
    voices = voices && voices.length ? voices : speechSynthesis.getVoices();
    const want = lang === 'de' ? 'de' : 'en';
    return voices.find(v => v.lang.toLowerCase().startsWith(want) && /male|stefan|david|mark|george|daniel/i.test(v.name)) ||
      voices.find(v => v.lang.toLowerCase().startsWith(want)) || null;
  }
  const queue = [], recent = new Map();
  let speaking = false;
  function nextVoice() {
    if (speaking || !queue.length) return;
    const item=queue.shift();
    if(Date.now()-item.created>12000){nextVoice();return;}
    const u=new SpeechSynthesisUtterance(item.text);
    u.lang=item.lang==='de'?'de-DE':'en-US'; const v=pickVoice(item.lang);if(v)u.voice=v;
    u.pitch=item.id.startsWith('COMP')?0.85:0.7;u.rate=1.03;u.volume=settings.speech;
    speaking=true;ducked=true;mix();
    let finished=false;
    const done=()=>{if(finished)return;finished=true;clearTimeout(watchdog);speaking=false;ducked=false;mix();nextVoice();};
    u.onend=done;u.onerror=done;const watchdog=setTimeout(done,Math.max(4000,item.text.length*120));
    speechSynthesis.speak(u);
  }
  S.say = function(id,lang,urgent) {
    const line=LINES[id];if(!line)return 0;
    const text=line[lang==='de'?1:0], now=Date.now();
    if(now-(recent.get(id)||0)<3500)return 0;recent.set(id,now);
    window.dispatchEvent(new CustomEvent('zonebots-radio',{detail:{text,urgent:!!urgent,computer:id.startsWith('COMP')}}));
    if(!settings.enabled || !settings.speech || !window.speechSynthesis)return 0;
    const item={id,lang,text,created:now};
    if(urgent)queue.unshift(item);else queue.push(item);
    if(queue.length>4)queue.pop();nextVoice();
    return 0.35+text.length*0.065;
  };
})();
