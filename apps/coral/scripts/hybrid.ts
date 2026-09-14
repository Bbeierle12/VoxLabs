import { runBench, BENCH_CASES, formatReport } from '../src/audio/choir-bench';
import { StreamingStft } from '../src/audio/streaming-stft';
import { NoteDetector } from '../src/audio/note-detector';
import { DETECTOR_CONFIG } from '../src/config/detector';
const SR=44100, N=8192, BIN=SR/N;
const midiToHz=(m:number)=>440*Math.pow(2,(m-69)/12);
/** Harmonic-family gate (from CHA v1.2.1): a note at k·f0 of an accepted lower note stays only if its own
 *  fundamental AND its own partial series sit ≥ margin dB above the parent's fitted harmonic envelope. */
function familyGate(mags:Float32Array, midis:number[], margin=10, tolCents=28){
  const db=(b:number)=>20*Math.log10((mags[b]??0)+1e-12);
  const peakNear=(f:number)=>{ const x=f/BIN; const lo=Math.max(1,Math.floor(x*(1-0.0175))), hi=Math.min(mags.length-2,Math.ceil(x*(1+0.0175))); let pk=-1; for(let i=lo;i<=hi;i++){ if(mags[i]!>mags[i-1]!&&mags[i]!>=mags[i+1]!&&(pk<0||mags[i]!>mags[pk]!)) pk=i; } return pk<0?null:{f:pk*BIN,amp:db(pk)}; };
  const parts=(f0:number,kmax:number)=>{ const out:{k:number,amp:number}[]=[]; for(let k=1;k<=kmax;k++){ const p=peakNear(f0*k); if(p&&p.amp>-90) out.push({k,amp:p.amp}); } return out; };
  const sorted=[...midis].sort((a,b)=>a-b); const accepted:{midi:number,f0:number,parts:{k:number,amp:number}[]}[]=[];
  for(const m of sorted){
    const f0=midiToHz(m); const own=parts(f0,3); if(!own.length){ accepted.push({midi:m,f0,parts:parts(f0,12)}); continue; }
    let grouped=false;
    for(const p of accepted){
      const r=f0/p.f0, k=Math.round(r); if(k<2||k>12) continue; if(Math.abs(1200*Math.log2(r/k))>tolCents) continue;
      // parent envelope: LS fit of dB vs log2(k) over parent partials not at multiples of k, dropping foreign (above-trend) points
      let pts=p.parts.filter(q=>q.k%k!==0).map(q=>[Math.log2(q.k),q.amp] as [number,number]);
      const fit=(P:[number,number][])=>{ let sx=0,sy=0,sxx=0,sxy=0; for(const [x,y] of P){sx+=x;sy+=y;sxx+=x*x;sxy+=x*y;} const n=P.length; const slope=n>1?(n*sxy-sx*sy)/(n*sxx-sx*sx||1):-6; const icpt=n>1?(sy-slope*sx)/n:(P[0]?.[1]??-60); return {slope:Math.min(0,slope),icpt}; };
      let f=fit(pts); for(let it=0;it<2&&pts.length>2;it++){ const keep=pts.filter(([x,y])=>y-(f.icpt+f.slope*x)<=4); if(keep.length===pts.length||keep.length<2) break; pts=keep; f=fit(pts); }
      const env=(kk:number)=>f.icpt+f.slope*Math.log2(kk);
      const excess=own.map(q=>q.amp-env(k*q.k)); const mean=excess.reduce((a,b)=>a+b,0)/excess.length;
      if(!(excess[0]!>=margin && mean>=margin)){ grouped=true; break; }
    }
    if(!grouped) accepted.push({midi:m,f0,parts:parts(f0,12)});
  }
  return new Set(accepted.map(a=>a.midi));
}
function mk(gate:boolean){ return (samples:Float32Array)=>{ const stft=new StreamingStft(N,2048); const det=new NoteDetector({numBins:N/2,sampleRate:SR,minFreqHz:50,maxFreqHz:8000,harmonic:true,harmonicCount:DETECTOR_CONFIG.harmonicCount,salienceThreshold:DETECTOR_CONFIG.salienceThreshold,decayPerFrame:DETECTOR_CONFIG.decayPerFrame}); let last=new Set<number>(); let mags:Float32Array|null=null; for(let o=0;o<samples.length;o+=128){ for(const m of stft.pushSamples(samples.subarray(o,o+128))){ last=det.analyze(m); mags=m; } } return gate&&mags?familyGate(mags,[...last]):last; }; }
function tone(f0:number, ampsDb:number[], noiseDb=-120, sec=0.6){ const n=Math.floor(SR*sec); const s=new Float32Array(n); let ph=0; for(let i=0;i<n;i++){ ph+=2*Math.PI*f0/SR; let v=0; ampsDb.forEach((a,k)=>{v+=Math.pow(10,a/20)*Math.sin((k+1)*ph);}); v+=Math.pow(10,noiseDb/20)*(Math.random()*2-1); s[i]=v*0.3;} return s; }
const singles:[string,Float32Array,number[]][]=[['baritone A2 chest',tone(110,[-20,-8,-9,-16,-12,-22,-26,-30,-34,-38]),[45]],['baritone D3',tone(146.8,[-14,-6,-12,-10,-18,-24,-28,-32]),[50]],['G3 strong H2',tone(196,[-16,-4,-10,-12,-14,-20,-24,-28]),[55]],['breath + baritone',tone(110,[-20,-8,-9,-16,-12,-22,-26,-30],-35),[45]],['bass C3 + sop C5',(()=>{const a=tone(130.8,[-10,-14,-18,-24,-28,-32,-36,-40]); const b=tone(523.25,[-8,-17,-26,-35]); const s=new Float32Array(a.length); for(let i=0;i<s.length;i++) s[i]=a[i]!+b[i]!; return s;})(),[48,72]]];
for(const gate of [false,true]){ const det=mk(gate); const rep=runBench(BENCH_CASES,det); console.log(`\n${gate?'Coral + family gate':'Coral alone     '}: chord battery F1=${rep.overall.f1.toFixed(3)} P=${rep.overall.precision.toFixed(3)} R=${rep.overall.recall.toFixed(3)}  octave-dbl F1=${rep.byTag['octave-dbl']!.f1.toFixed(3)} dbl-2oct F1=${rep.byTag['dbl-2oct']!.f1.toFixed(3)} high F1=${rep.byTag['high']!.f1.toFixed(3)}`); for(const [n,s,gt] of singles){ const d=[...det(s)].sort((a,b)=>a-b); console.log('   ',n.padEnd(20),'gt',gt,'->',d, d.length===gt.length&&gt.every(g=>d.includes(g))?'OK':'MISMATCH'); } }
