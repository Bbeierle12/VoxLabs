/* ============================================================
   Choral Harmony Analyzer — analysis module (no DOM, no audio)
   Runs identically in the browser and in Node (offline harness).
   v1.2.1: harmonic-family grouping (one singer → one card), peak-based voicedness,
         bass-anchored targets, target profiles, drift memory, tolerance bands, scatter.
   ============================================================ */

export const NOTE_NAMES = ['C','C♯','D','E♭','E','F','F♯','G','A♭','A','B♭','B'];
export const PARTS = [
  {id:'B',name:'Bass',   lo:55,  hi:330,  rangeTxt:'B1 – E4 · 55–330 Hz'},
  {id:'T',name:'Tenor',  lo:120, hi:450,  rangeTxt:'C3 – A4 · 120–450 Hz'},
  {id:'A',name:'Alto',   lo:170, hi:700,  rangeTxt:'F3 – F5 · 170–700 Hz'},
  {id:'S',name:'Soprano',lo:250, hi:1100, rangeTxt:'C4 – C6 · 250–1100 Hz'},
];
PARTS.forEach(p=>{p.center=Math.sqrt(p.lo*p.hi);p.span=Math.log2(p.hi/p.lo)/2;});
export const FMIN=50, FMAX=1300;

/* ---------- configuration (mutated by the UI) ---------- */
export const cfg = {
  a4:440,
  sections:{B:true,T:true,A:true,S:true},
  sup:3,            // overtone suppression 1..5
  thr:-82,          // pitch threshold dB
  profile:'ensemble', // target profile: pure | ensemble | equal
  vibratoHeavy:false, // widens the flat-side tolerance by 10 c
};

/* ---------- target profiles: cents above the chord root for interval classes 0..11 ----------
   pure     = 5-limit just intonation (7:4 for the minor seventh)
   ensemble = centre of what measured a cappella ensembles produce and expert listeners prefer
              (memo 2: M3 392–405 measured, 395 preferred; m3 ≈ ET; P5 702–706; m7 977–985)
   equal    = 12-TET                                                                     */
export const PROFILES = {
  pure:     {name:'Pure (just)',    c:[0,112,204,316,386,498,583,702,814,884,969,1088]},
  ensemble: {name:'Ensemble',       c:[0,108,202,302,394,499,590,703,806,898,985,1092]},
  equal:    {name:'Equal (12-TET)', c:[0,100,200,300,400,500,600,700,800,900,1000,1100]},
};

/* ---------- pitch math ---------- */
export const hzToMidi=f=>69+12*Math.log2(f/cfg.a4);
export const midiToHz=m=>cfg.a4*Math.pow(2,(m-69)/12);
export const cents=(f1,f2)=>1200*Math.log2(f2/f1);
export function noteInfo(f){
  const m=hzToMidi(f), r=Math.round(m);
  return {midi:r, pc:((r%12)+12)%12, name:NOTE_NAMES[((r%12)+12)%12], oct:Math.floor(r/12)-1, cents:(m-r)*100, etHz:midiToHz(r)};
}

/* Interval classes: pure ratio (for beat maths), name, consonance weight */
export const JI = {
  0:{p:1,q:1,name:'Unison',w:1.0}, 1:{p:16,q:15,name:'m2',w:0.15}, 2:{p:9,q:8,name:'M2',w:0.3},
  3:{p:6,q:5,name:'m3',w:0.7}, 4:{p:5,q:4,name:'M3',w:0.7}, 5:{p:4,q:3,name:'P4',w:0.8},
  6:{p:7,q:5,name:'TT',w:0.2}, 7:{p:3,q:2,name:'P5',w:0.9}, 8:{p:8,q:5,name:'m6',w:0.65},
  9:{p:5,q:3,name:'M6',w:0.65}, 10:{p:7,q:4,name:'m7',w:0.35}, 11:{p:15,q:8,name:'M7',w:0.15},
};
export const CHORDS = [
  {name:'Major',      iv:[0,4,7],    ints:[4,5,6]},
  {name:'Minor',      iv:[0,3,7],    ints:[10,12,15]},
  {name:'Dom 7',      iv:[0,4,7,10], ints:[4,5,6,7]},
  {name:'Major 7',    iv:[0,4,7,11], ints:[8,10,12,15]},
  {name:'Minor 7',    iv:[0,3,7,10], ints:[10,12,15,18]},
  {name:'Dim',        iv:[0,3,6],    ints:[5,6,7]},
  {name:'Sus4',       iv:[0,5,7],    ints:[6,8,9]},
  {name:'Sus2',       iv:[0,2,7],    ints:[8,9,12]},
  {name:'Open 5th',   iv:[0,7],      ints:[2,3]},
  {name:'Major 3rd',  iv:[0,4],      ints:[4,5]},
  {name:'Minor 3rd',  iv:[0,3],      ints:[5,6]},
  {name:'Perfect 4th',iv:[0,5],      ints:[3,4]},
  {name:'Unison/8ve', iv:[0],        ints:[1]},
];
CHORDS.forEach(c=>{c.ratio=c.ints.join(':');});
export const gcd=(a,b)=>b?gcd(b,a%b):a;
export const redTxt=(p,q,sep='/')=>{const g=gcd(p,q);return `${p/g}${sep}${q/g}`;};

export function identifyChord(voices){
  if(!voices.length) return null;
  const pcs=[...new Set(voices.map(v=>v.pc))];
  let best=null;
  for(const c of CHORDS){
    for(const root of pcs){
      const set=c.iv.map(i=>(root+i)%12);
      const covered=pcs.filter(p=>set.includes(p)).length;
      const missing=c.iv.length-set.filter(p=>pcs.includes(p)).length;
      if(covered!==pcs.length) continue;
      const score=covered*10 - missing*4 + (voices[0].pc===root?2:0) - c.iv.length*0.1;
      if(!best||score>best.score) best={chord:c,root,score,missing};
    }
  }
  if(!best) return {chord:{name:'Cluster / Polyphony',ratio:'—',iv:[],ints:[]},root:voices[0].pc,cluster:true};
  return best;
}

/* ---------- targets: anchored on the sounding bass, sized by the active profile ----------
   memo 2 §5: singers tune to the bass (Devaney 2012; BHS); the bass never gets a "move" instruction.
   Returns per voice: {tgt, jiCents (target offset from the voice's ET pitch), err (singer − target),
   phi (profile offset from ET for drift), alt:{pure,equal} marker offsets, anchor:boolean}        */
export function targets(voices,id){
  if(!id||id.cluster) return voices.map(()=>null);
  const prof=PROFILES[cfg.profile].c;
  const cls=voices.map(v=>{const k=id.chord.iv.indexOf(((v.pc-id.root)%12+12)%12); return k<0?null:id.chord.iv[k];});
  const ai=cls.findIndex(c=>c!=null);           // lowest chord-tone voice is the anchor
  if(ai<0) return voices.map(()=>null);
  const A=voices[ai], clsA=cls[ai];
  const rootHz=A.f*Math.pow(2,-prof[clsA]/1200);
  const rootHzOf=p=>A.f*Math.pow(2,-PROFILES[p].c[clsA]/1200);
  return voices.map((v,i)=>{
    const c=cls[i]; if(c==null) return null;
    const k=id.chord.iv.indexOf(c);
    const near=(hz)=>{ let t=hz; while(t/v.f>Math.SQRT2) t/=2; while(v.f/t>Math.SQRT2) t*=2; return t; };
    const tgt=near(rootHz*Math.pow(2,prof[c]/1200));
    const alt={}; for(const p of ['pure','equal']) alt[p]=cents(v.etHz, near(rootHzOf(p)*Math.pow(2,PROFILES[p].c[c]/1200)));
    return {tgt, jiCents:cents(v.etHz,tgt), err:i===ai?0:cents(tgt,v.f), phi:prof[c]-100*c, alt, anchor:i===ai,
            k, int:id.chord.ints[k], role:k===0?'root':(JI[c]||{}).name, ratioTxt:redTxt(id.chord.ints[k],id.chord.ints[0])};
  });
}

/* pair analysis: interval class, pure ratio for beats, cents, error against the profile target, beat rate */
export function pairAnalysis(voices,tg){
  const out=[];
  for(let i=0;i<voices.length;i++)for(let j=i+1;j<voices.length;j++){
    const a=voices[i],b=voices[j];
    const c=cents(a.f,b.f);
    const semis=Math.round(c/100); const k=((semis%12)+12)%12; const oct=Math.floor(semis/12);
    const ji=JI[k];
    let P,Q,pureC;
    const ta=tg&&tg[i], tb=tg&&tg[j];
    if(ta&&tb){
      const n=Math.round(Math.log2((tb.tgt/ta.tgt)*ta.int/tb.int));
      P=tb.int; Q=ta.int; if(n>0) P*=Math.pow(2,n); else Q*=Math.pow(2,-n);
      pureC=cents(ta.tgt,tb.tgt);                       // profile-consistent target size
    } else { P=ji.p*Math.pow(2,oct); Q=ji.q; pureC=1200*Math.log2(P/Q); }
    const g=gcd(P,Q); P/=g; Q/=g;
    const err=c-pureC;
    const beat=Math.abs(Q*b.f-P*a.f);
    const tune=Math.exp(-Math.pow(err/20,2));
    out.push({a,b,cls:k,name:ji.name+(oct?'+'+oct+'8ve':''),ratio:`${P}:${Q}`,cents:c,err,beat,tune,w:ji.w});
  }
  return out;
}
export function consonanceIndex(pairs){
  if(!pairs.length) return null;
  const tune=pairs.reduce((t,p)=>t+p.tune,0)/pairs.length;
  const w=pairs.reduce((t,p)=>t+p.w,0)/pairs.length;
  return Math.round(100*(0.6*tune+0.4*w));
}

/* ---------- tolerance bands (memo 2 §5 rule 4) ----------
   ±10 c in tune, 10–20 marginal, >20 out; flat side +10 c when vibrato-heavy; ±20/30 for notes < 250 ms */
export function band(err,heldMs){
  let tin=10, tout=20;
  if(heldMs!=null&&heldMs<250){ tin=20; tout=30; }
  else if(cfg.vibratoHeavy&&err<0){ tin=20; tout=30; }
  const a=Math.abs(err);
  return a<=tin?'in':a<=tout?'marginal':'out';
}

/* ---------- SATB assignment (order-preserving min-cost) ---------- */
export const HOLD_MS=400;
export const ONSET_FRAMES=4;
export const cards=PARTS.map(()=>({f:null,hist:[],last:0,active:false,hits:0,since:0,scat:[],scatter:null}));
export function resetCards(){ cards.forEach(c=>{c.active=false;c.f=null;c.hist=[];c.hits=0;c.since=0;c.scat=[];c.scatter=null;}); }
function partCost(p,f,pi){
  if(!cfg.sections[p.id]||f<p.lo||f>p.hi) return 1e6;
  let c=Math.pow(Math.log2(f/p.center)/p.span,2);
  const card=cards[pi]; if(card&&card.hits>0&&card.f&&Math.abs(cents(card.f,f))<60) c-=0.6;
  return c;
}
export function assignParts(freqs){
  const n=freqs.length; let best=null;
  const rec=(i,minPart,acc,cost)=>{
    if(cost>=(best?best.cost:1e9)) return;
    if(i===n){best={cost,acc:[...acc]};return;}
    for(let p=minPart;p<4;p++){const c=partCost(PARTS[p],freqs[i],p); acc.push(p); rec(i+1,p+1,acc,cost+c); acc.pop();}
    acc.push(-1); rec(i+1,minPart,acc,cost+1.1); acc.pop();
  };
  rec(0,0,[],0);
  return best?best.acc:freqs.map(()=>-1);
}
export function updateCards(freqs,assign,now){
  const seen=new Set();
  freqs.forEach((f,i)=>{ const p=assign[i]; if(p<0) return; seen.add(p);
    const c=cards[p];
    if(c.f&&Math.abs(cents(c.f,f))>80){ c.hist=[]; c.since=now; c.scat=[]; }   // new note
    if(!c.hist.length) c.since=now;
    c.hist.push(f); if(c.hist.length>5) c.hist.shift();
    const s=[...c.hist].sort((a,b)=>a-b); c.f=s[Math.floor(s.length/2)]; c.last=now; c.hits++; if(c.hits>=ONSET_FRAMES) c.active=true; });
  cards.forEach((c,p)=>{
    if(seen.has(p)) return;
    const dup=cards.some((o,q)=>q!==p&&seen.has(q)&&c.f&&o.f&&Math.abs(cents(c.f,o.f))<40);
    if(dup||now-c.last>HOLD_MS){c.active=false;c.f=null;c.hist=[];c.hits=0;c.since=0;c.scat=[];c.scatter=null;}
  });
}

/* ---------- peak picking + harmonic-family grouping ----------
   Every sung tone is a harmonic family: a parent fundamental f0 and its partials k·f0. Only the parent
   lights a card; the partials stay in the family (and stay visible in the visualisers). A partial leaves
   the family and becomes its own voice only when its level is implausible for a harmonic of that parent:
   more than `margin` dB above BOTH neighbouring partials of the parent (a vocal-tract formant can lift one
   harmonic ~10 dB above its neighbours; a second singer on that pitch usually does far more). The
   Overtone Suppression slider sets that margin.                                                          */
export function detectVoices(spec,binHz){
  const n=spec.length; const iMin=Math.max(2,Math.floor(FMIN/binHz)), iMax=Math.min(n-3,Math.ceil(FMAX/binHz));
  const tmp=[]; for(let i=iMin;i<iMax;i+=3) tmp.push(spec[i]); tmp.sort((a,b)=>a-b);
  const floor=tmp[Math.floor(tmp.length*0.5)]||-100;
  const thr=Math.max(cfg.thr, floor+12);
  const at=f=>{const x=f/binHz, i=Math.floor(x); if(i<1||i>=n-1) return -120; return spec[i]+(spec[i+1]-spec[i])*(x-i);};
  // local median (±150 Hz) so "peak" means "stands above its surroundings", not "above the global floor"
  const halfW=Math.max(3,Math.round(150/binHz));
  const localMed=i=>{ const lo=Math.max(0,i-halfW), hi=Math.min(n-1,i+halfW); const arr=[]; for(let j=lo;j<=hi;j+=2) arr.push(spec[j]); arr.sort((x,y)=>x-y); return arr[arr.length>>1]; };
  // a partial of f at multiple m is "present" if there is a local maximum within ±30 c that stands ≥8 dB above its local median
  const partialPeak=(f,m)=>{ const x=f*m/binHz; const lo=Math.max(1,Math.floor(x*(1-0.0175))), hi=Math.min(n-2,Math.ceil(x*(1+0.0175))); let pk=-1;
    for(let i=lo;i<=hi;i++) if(spec[i]>spec[i-1]&&spec[i]>=spec[i+1]&&(pk<0||spec[i]>spec[pk])) pk=i;
    if(pk<0) return null; const amp=spec[pk]; if(amp<localMed(pk)+8||amp<floor+6) return null; return {f:pk*binHz,amp,k:m}; };
  // candidate fundamentals: spectral peaks above threshold that stand out locally
  const cands=[];
  for(let i=iMin;i<iMax;i++){
    const b=spec[i]; if(b<thr) continue;
    if(b>spec[i-1]&&b>=spec[i+1]&&b>spec[i-2]&&b>=spec[i+2]){
      if(b<localMed(i)+8) continue;                                  // noise: not a peak relative to its surroundings
      const a=spec[i-1],g=spec[i+1]; const d=(a-g)/(2*(a-2*b+g))||0;
      cands.push({f:(i+d)*binHz, amp:b-0.25*(a-g)*d, i});
    }
  }
  cands.sort((x,y)=>x.f-y.f);
  const merged=[]; for(const c of cands){const l=merged[merged.length-1]; if(l&&Math.abs(cents(l.f,c.f))<25){ if(c.amp>l.amp) merged[merged.length-1]=c; } else merged.push(c);}
  // voicedness: a sung tone shows at least two of partials 2..6 as real peaks (breath, consonants and room noise do not)
  for(const c of merged){ c.parts=[]; for(let m=2;m<=6;m++){ const p=partialPeak(c.f,m); if(p) c.parts.push(p); }
    let s=0; for(let m=2;m<=4;m++) s+=Math.max(-100,at(c.f*m))+100; c.hps=(c.amp+100)+0.35*s; }
  const voiced=merged.filter(c=>c.parts.length>=2);
  // family grouping. A partial at k·f0 leaves the family only when the CANDIDATE'S OWN partial series
  // (k, 2k, 3k of the parent) sits consistently above the parent's harmonic envelope — a second singer
  // brings a whole series that is too strong at every slot, a formant lifts one slot at most.
  const tol=[0,40,34,28,22,16][cfg.sup]; const margin=[0,16,13,10,7,4][cfg.sup];
  const accepted=[], suppressed=[];
  const parentEnvelope=(p,k0)=>{ // fit of the parent's partial levels (dB) vs log2(k), excluding multiples of k0.
    // Other voices only ever ADD energy to a slot, so points sitting well above the trend are foreign: drop them and refit.
    let pts=[[0,p.amp]]; for(const q of p.parts) if(q.k%k0!==0) pts.push([Math.log2(q.k),q.amp]);
    const fit=P=>{ let sx=0,sy=0,sxx=0,sxy=0; for(const [x,y] of P){sx+=x;sy+=y;sxx+=x*x;sxy+=x*y;} const n=P.length;
      const slope=n>1?(n*sxy-sx*sy)/(n*sxx-sx*sx||1):-6, icpt=n>1?(sy-slope*sx)/n:p.amp; return {slope:Math.min(0,slope),icpt}; };
    let f=fit(pts);
    for(let it=0;it<2&&pts.length>2;it++){ const keep=pts.filter(([x,y])=>y-(f.icpt+f.slope*x)<=4); if(keep.length===pts.length||keep.length<2) break; pts=keep; f=fit(pts); }
    return k=>f.icpt+f.slope*Math.log2(k);
  };
  for(const c of voiced){
    let parent=null;
    for(const p of accepted){
      const r=c.f/p.f; const k=Math.round(r); if(k<2||k>12) continue;
      if(Math.abs(1200*Math.log2(r/k))>tol) continue;
      const env=parentEnvelope(p,k);
      const slots=[{k:1,amp:c.amp},...c.parts.filter(q=>q.k<=3)];       // candidate's own 1st..3rd partials
      const excess=slots.map(q=>q.amp-env(k*q.k));
      const mean=excess.reduce((a,b)=>a+b,0)/excess.length;
      const independent = excess[0]>=margin && mean>=margin;   // its own fundamental AND its series must tower over the parent's envelope
      c.excess=mean;
      if(!independent){ parent=p; break; }
    }
    if(parent){ c.over=true; c.parent=parent; parent.members.push(c); suppressed.push(c); }
    else { c.members=[]; accepted.push(c); }
  }
  // number of voices comes from the evidence, capped by the sections that are switched on
  accepted.sort((x,y)=>y.hps-x.hps);
  const maxV=Object.values(cfg.sections).filter(Boolean).length||1;
  const keep=accepted.slice(0,maxV).sort((x,y)=>x.f-y.f);
  return {voices:keep, suppressed, floor};
}

/* ---------- section scatter from the width of a high partial (memo 3 §3 stage 1) ----------
   Energy-weighted RMS width (in bins) of the highest isolated partial k ≤ 10, with the analysis
   window's own RMS width removed in quadrature; converted to cents at k·f0. Returns {cents,k} or null. */
const HANN_SIGMA_BINS=(()=>{ // RMS width of the Hann power main lobe, sampled at bin spacing
  let num=0,den=0; const D=b=>Math.abs(b)<1e-9?1:Math.sin(Math.PI*b)/(Math.PI*b);
  for(let b=-3;b<=3;b+=0.01){ const W=0.5*D(b)+0.25*(D(b-1)+D(b+1)); const p=W*W; num+=p*b*b; den+=p; }
  return Math.sqrt(num/den);
})();
export function sectionScatter(spec,binHz,f0,otherF0s,floor){
  const n=spec.length;
  const isolated=fk=>otherF0s.every(o=>{ const m=Math.round(fk/o); return m<1||Math.abs(1200*Math.log2(fk/(m*o)))>50; });
  for(let k=10;k>=2;k--){
    const fk=k*f0; if(fk>binHz*(n-4)) continue; if(fk<500) break;  /* below ~500 Hz the window dominates: no estimate */
    if(!isolated(fk)) continue;
    const x0=fk/binHz; const half=Math.max(3, x0*(Math.pow(2,60/1200)-1));
    const lo=Math.max(1,Math.floor(x0-half)), hi=Math.min(n-2,Math.ceil(x0+half));
    let pk=lo; for(let i=lo;i<=hi;i++) if(spec[i]>spec[pk]) pk=i;
    if(spec[pk]<floor+20) continue;                       // too close to the noise floor to measure a width
    // integrate only the contiguous region within 12 dB of the peak, so the noise floor cannot inflate the moment
    const cut=Math.max(spec[pk]-12, floor+6);
    let a=pk; while(a>lo&&spec[a-1]>cut) a--; let b=pk; while(b<hi&&spec[b+1]>cut) b++;
    const pf=Math.pow(10,cut/10);
    let num=0,den=0,mean=0;
    for(let i=a;i<=b;i++){ const p=Math.max(0,Math.pow(10,spec[i]/10)-pf); den+=p; mean+=p*i; }
    if(den<=0) continue; mean/=den;
    for(let i=a;i<=b;i++){ const p=Math.max(0,Math.pow(10,spec[i]/10)-pf); num+=p*(i-mean)*(i-mean); }
    const sigBins=Math.sqrt(Math.max(0,num/den-HANN_SIGMA_BINS*HANN_SIGMA_BINS));
    const sigHz=sigBins*binHz;
    return {cents:1200*Math.log2(1+sigHz/(mean*binHz)), k};
  }
  return null;
}
export const scatterBand=c=>c==null?null:c<10?'tight':c<15?'typical':c<30?'loose':'scattered';

/* ---------- drift memory (memo 2 §5 rule 3) ----------
   τ* = mean over chord tones of (ET deviation − profile offset); r ← 0.85·r + 0.15·τ* per chord change
   (or every 2 s while a chord is held); "drift since start" = r − r0.                                  */
export const drift={r:null,r0:null,lastKey:null,lastT:0,tau:null};
export function resetDrift(){ drift.r=null; drift.r0=null; drift.lastKey=null; drift.lastT=0; drift.tau=null; }
export function driftStep(voices,tg,chordKey,now){
  const vals=[]; voices.forEach((v,i)=>{ const t=tg[i]; if(t) vals.push(v.cents-t.phi); });
  if(!vals.length) return;
  const tau=vals.reduce((a,b)=>a+b,0)/vals.length; drift.tau=tau;
  const due=chordKey!==drift.lastKey || now-drift.lastT>2000;
  if(!due) return;
  drift.lastKey=chordKey; drift.lastT=now;
  if(drift.r==null){ drift.r=tau; drift.r0=tau; } else drift.r=0.85*drift.r+0.15*tau;
}
export const driftSinceStart=()=>drift.r==null?null:drift.r-drift.r0;

/* ---------- chord hold tracking (memo 2 §5 rule 6) ---------- */
export const hold={key:null,start:0};
export const CHORD_HOLD_MS=300;

/* ---------- one full frame: spectrum → everything the UI needs ---------- */
export function analyzeFrame(spec,binHz,now){
  const det=detectVoices(spec,binHz);
  const assign=assignParts(det.voices.map(v=>v.f));
  updateCards(det.voices.map(v=>v.f),assign,now);
  const voices=[]; const assignByPart=[null,null,null,null];
  cards.forEach((c,pi)=>{ if(c.active&&c.f!=null){ const ni=noteInfo(c.f); voices.push({f:c.f,part:pi,heldMs:now-c.since,...ni}); } });
  voices.sort((a,b)=>a.f-b.f); voices.forEach((v,i)=>assignByPart[v.part]=i);
  // scatter per active card (median of last 5 frames)
  const f0s=voices.map(v=>v.f);
  voices.forEach(v=>{ const c=cards[v.part]; const s=sectionScatter(spec,binHz,v.f,f0s.filter(o=>o!==v.f),det.floor);
    if(s){ c.scat.push(s.cents); if(c.scat.length>5) c.scat.shift(); const so=[...c.scat].sort((a,b)=>a-b); c.scatter=so[Math.floor(so.length/2)]; c.scatterK=s.k; }
    v.scatter=c.scatter; v.scatterK=c.scatterK; v.scatterBand=scatterBand(c.scatter); });
  const id=identifyChord(voices);
  const key=id?(id.cluster?'cluster':id.root+':'+id.chord.name+':'+voices.length):null;
  if(key!==hold.key){ hold.key=key; hold.start=now; }
  const chordHeldMs=key?now-hold.start:0;
  const tg=targets(voices,id);
  const pairs=pairAnalysis(voices,tg);
  const cons=consonanceIndex(pairs);
  if(id&&!id.cluster&&chordHeldMs>=CHORD_HOLD_MS) driftStep(voices,tg,key,now);
  const counts={B:0,T:0,A:0,S:0}; voices.forEach(v=>counts[PARTS[v.part].id]++);
  return {det,assign,voices,assignByPart,id,tg,pairs,cons,counts,chordHeldMs,settled:chordHeldMs>=CHORD_HOLD_MS,drift:driftSinceStart(),tau:drift.tau};
}

/* ---------- presets & synthetic spectra ---------- */
const P=(n,o,c=0)=>({n,o,c});
export const PRESETS={
  cmaj:   {title:'Pure C Major Just Triad (4:5:6)',            v:[P('C',3),P('G',3,2.0),P('E',4,-13.7),P('C',5)]},
  cmajEns:{title:'Ensemble-tuned C Major (M3 394 c)',          v:[P('C',3),P('G',3,3.0),P('E',4,-6.0),P('C',5)]},
  g7:     {title:'G Dominant 7th Cadence (4:5:6:7)',           v:[P('G',2),P('D',4,2.0),P('F',4,-31.2),P('B',4,-13.7)]},
  sharp3: {title:'Out-of-Tune Sharp 3rd Dyad',                 v:[P('C',3),P('E',4,+22)]},
  flatBass:{title:'Whole chord 30 c flat (drift test)',        v:[P('C',3,-30),P('G',3,-28),P('E',4,-43.7),P('C',5,-30)]},
  tallis: {title:'Tallis-style Renaissance Polyphony (G–D–B–G)',v:[P('G',2),P('D',3,2.0),P('B',3,-13.7),P('G',4)]},
  amin:   {title:'A Minor Triad (10:12:15)',                   v:[P('A',2),P('E',3,2.0),P('C',4,15.6),P('A',4)]},
};
export function presetFreqs(key){ return PRESETS[key].v.map(x=>{const m=12*(x.o+1)+NOTE_NAMES.indexOf(x.n); return midiToHz(m)*Math.pow(2,x.c/1200);}); }
/* tones: numbers (Hz) or {f, amp (dB, default −8), amps:[per-harmonic dB], noise:dB (broadband level)} */
export function synthSpectrum(tones,binHz,n,opts={}){
  const base=opts.noise??-95;
  const spec=new Float32Array(n).fill(base);
  for(let i=0;i<n;i++) spec[i]+=(Math.random()-0.5)*(opts.noiseSpread??3);
  for(const t of tones){ const f=typeof t==='number'?t:t.f, a0=typeof t==='number'?-8:(t.amp??-8); const amps=typeof t==='object'&&t.amps;
    for(let h=1;h<=(amps?amps.length:8);h++){ const fh=f*h, a=amps?amps[h-1]:a0-9*(h-1); const x=fh/binHz; const w=1.6;
      for(let i=Math.max(0,Math.floor(x-4));i<Math.min(n,x+5);i++){ const d=(i-x)/w; const v=a-8*d*d; spec[i]=10*Math.log10(Math.pow(10,spec[i]/10)+Math.pow(10,v/10)); } } }
  return spec;
}
