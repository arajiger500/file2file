(() => {
  const engine = document.querySelector('.engine-traveller');
  const rig = document.querySelector('.gear-rig');
  const cogs = document.querySelectorAll('.cog');
  const rail = document.querySelector('.scroll-rail span');
  const local = document.querySelector('.local');
  const motionButton = document.querySelector('.motion-toggle');
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
  const finePointer = window.matchMedia('(hover: hover) and (pointer: fine)');
  let enabled = !reducedMotion.matches;
  let tickId = 0, angle = 0, lastTime = 0, smoothedY = window.scrollY;
  let pointerX = 0, pointerY = 0, targetX = 0, targetY = 0;
  let hovering = false;
  let localCenter = 0;
  function measure(){localCenter=local.offsetTop-window.innerHeight*.28;}
  measure();
  window.addEventListener('resize',measure,{passive:true});
  const clamp = (n,a,b) => Math.max(a,Math.min(b,n));
  function progress(){return clamp(window.scrollY / Math.max(1,document.documentElement.scrollHeight-window.innerHeight),0,1);}
  function pose(y){
    const mobile = window.innerWidth <= 620;
    const phase = y / Math.max(1,window.innerHeight);
    const travel = Math.sin(phase*.78);
    const approach = .94 + .26*Math.sin(phase*.5);
    engine.style.transform = `translate3d(${travel*(mobile?55:135)+pointerX*14}px,${-Math.sin(phase*.9)*100+pointerY*9}px,0) scale(${approach}) rotate(${travel*-6}deg)`;
    const distance = Math.abs(y-localCenter);
    const focus = 1-clamp(distance/window.innerHeight,0,1);
    engine.style.opacity = String(.85-focus*.28);
    rig.style.transform = `translate3d(${travel*-65}px,${-Math.sin(phase*.8)*70}px,0) scale(${1+focus*.35})`;
    rig.style.opacity = String(phase>1.2?(.25+focus*.65):.9);
    rail.style.transform = `scaleY(${progress()})`;
  }
  function frame(time){
    if (!enabled || document.hidden) { tickId=0;return; }
    const dt = Math.min((time-lastTime)/1000 || 0,.05); lastTime=time;
    smoothedY += (window.scrollY-smoothedY)*Math.min(1,dt*10);
    pointerX += (targetX-pointerX)*Math.min(1,dt*5);
    pointerY += (targetY-pointerY)*Math.min(1,dt*5);
    const scrollSpeed = Math.min(Math.abs(window.scrollY-smoothedY)*.05,18);
    angle = (angle+dt*(hovering?22:7)+dt*scrollSpeed)%360;
    cogs[0].style.transform = `rotate(${angle}deg)`;
    cogs[1].style.transform = `rotate(${10-angle}deg)`;
    pose(smoothedY);
    tickId=requestAnimationFrame(frame);
  }
  function start(){if(enabled&&!document.hidden&&!tickId){lastTime=performance.now();tickId=requestAnimationFrame(frame);}}
  function setEnabled(value){
    enabled=value; document.body.classList.toggle('motion-paused',!enabled);
    motionButton.setAttribute('aria-pressed',String(enabled));
    motionButton.textContent=enabled?'Motion on':'Motion off';
    if(!enabled){cancelAnimationFrame(tickId);tickId=0;engine.style.transform='none';rig.style.transform='none';}
    else start();
  }
  motionButton.addEventListener('click',()=>setEnabled(!enabled));
  reducedMotion.addEventListener('change',e=>setEnabled(!e.matches));
  document.addEventListener('visibilitychange',()=>{if(document.hidden){cancelAnimationFrame(tickId);tickId=0;}else start();});
  document.addEventListener('pointermove',e=>{if(!finePointer.matches)return;targetX=(e.clientX/window.innerWidth-.5)*2;targetY=(e.clientY/window.innerHeight-.5)*2;},{passive:true});
  document.addEventListener('pointerout',e=>{if(!e.relatedTarget){targetX=0;targetY=0;}});
  document.querySelectorAll('.button,.categories,.format-board').forEach(el=>{el.addEventListener('pointerenter',()=>hovering=true);el.addEventListener('pointerleave',()=>hovering=false);});
  window.addEventListener('scroll',()=>{rail.style.transform=`scaleY(${progress()})`;},{passive:true});
  document.querySelectorAll('#input-format,#output-format,.categories').forEach(el=>el.addEventListener('change',()=>{const desc=document.querySelector('.pair-description');desc.classList.add('changing');window.setTimeout(()=>desc.classList.remove('changing'),130);}));
  setEnabled(enabled);
})();
