// Native browser fullscreen. Browser shortcuts remain under browser control.
(function(global){
  'use strict';
  const doc=global.document;
  let initialized=false,busy=false,lastActive=false;
  let buttons=[],status=null,help=null,statusTimer=null;

  function isActive(){
    return Boolean(doc&&(doc.fullscreenElement||doc.webkitFullscreenElement));
  }
  function entry(){
    const root=doc&&doc.documentElement;
    if(!root)return null;
    if(typeof root.requestFullscreen==='function'){
      return {target:root,method:root.requestFullscreen,enabled:doc.fullscreenEnabled!==false};
    }
    if(typeof root.webkitRequestFullscreen==='function'){
      return {target:root,method:root.webkitRequestFullscreen,enabled:doc.fullscreenEnabled!==false&&doc.webkitFullscreenEnabled!==false};
    }
    return null;
  }
  function setStatus(message){
    if(status){status.textContent=message;status.hidden=!message;}
    if(statusTimer!==null&&typeof global.clearTimeout==='function')global.clearTimeout(statusTimer);
    if(message&&typeof global.setTimeout==='function')statusTimer=global.setTimeout(()=>{
      if(status)status.hidden=true;
      statusTimer=null;
    },4500);
  }
  function showHelp(message){
    setStatus(message);
    if(help)help.hidden=false;
  }
  function sync(){
    const active=isActive(),available=entry();
    if(doc&&doc.documentElement&&doc.documentElement.dataset){
      doc.documentElement.dataset.fullscreenSupported=String(Boolean(available&&available.enabled));
    }
    buttons.forEach(button=>{
      button.textContent=active?'Vollbild beenden':'Vollbild';
      button.setAttribute('aria-pressed',String(active));
      button.setAttribute('title',active?'Vollbild beenden (Esc)':'Spiel im Vollbild öffnen');
      button.setAttribute('aria-busy',String(busy));
    });
    if(active){
      setStatus('Vollbild aktiv. Mit Esc beenden.');
      if(help)help.hidden=true;
    }else if(lastActive){
      setStatus('Vollbild beendet.');
    }
    lastActive=active;
    return active;
  }
  async function toggle(){
    if(!doc||busy)return false;
    const exiting=isActive(),available=entry();
    let target,method;
    if(exiting){
      target=doc;
      method=typeof doc.exitFullscreen==='function'?doc.exitFullscreen:doc.webkitExitFullscreen;
      if(typeof method!=='function'){
        showHelp('Vollbild lässt sich hier nicht per Knopf beenden. Drücke Esc oder nutze den Vollbild-Knopf des Browsers.');
        return false;
      }
    }else{
      if(!available||!available.enabled){
        sync();
        showHelp(!available?
          'Dieser Browser bietet hier keinen Vollbildmodus an. Nutze die Anleitung für Browser-Vollbild.':
          'Dieser Browser erlaubt hier keinen Vollbildmodus. Nutze die Anleitung für Browser-Vollbild.');
        return false;
      }
      target=available.target;method=available.method;
    }
    busy=true;
    sync();
    try{
      // Invoke before the first await so the original click supplies user activation.
      const result=method.call(target);
      await result;
      const active=sync();
      if(exiting&&!active){
        setStatus('Vollbild beendet.');
      }else if(exiting&&active){
        showHelp('Vollbild konnte nicht beendet werden. Drücke Esc oder nutze den Vollbild-Knopf des Browsers.');
      }else if(!exiting&&!active){
        showHelp('Vollbild wurde nicht aktiviert. Nutze die Anleitung für Browser-Vollbild.');
      }
      return exiting?!active:active;
    }catch(error){
      sync();
      showHelp(exiting?
        'Vollbild konnte nicht beendet werden. Drücke Esc oder nutze den Vollbild-Knopf des Browsers.':
        'Vollbild konnte nicht gestartet werden. Nutze die Anleitung für Browser-Vollbild.');
      return false;
    }finally{
      busy=false;
      buttons.forEach(button=>button.setAttribute('aria-busy','false'));
    }
  }
  function init(){
    if(initialized||!doc)return;
    initialized=true;
    buttons=Array.from(doc.querySelectorAll('[data-fullscreen]'));
    status=doc.getElementById('fullscreenStatus');
    help=doc.getElementById('fullscreenHelp');
    buttons.forEach(button=>button.addEventListener('click',event=>{
      event.preventDefault();
      return toggle();
    }));
    doc.addEventListener('fullscreenchange',sync);
    doc.addEventListener('webkitfullscreenchange',sync);
    sync();
  }
  global.GameFullscreen=Object.freeze({init,toggle,isActive});
  init();
})(typeof window==='undefined'?globalThis:window);
