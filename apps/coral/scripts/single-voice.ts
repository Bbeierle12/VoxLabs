import { SpectrogramDsp } from '../src/audio/dsp-core';
const SR = 44100;
function run(samples: Float32Array){ const dsp=new SpectrogramDsp({fftSize:2048,hopSize:512,sampleRate:SR,minFreqHz:50,maxFreqHz:8000}); let last:number[]=[]; for(let o=0;o<samples.length;o+=128){const r=dsp.pushSamples(samples.subarray(o,o+128)); if(r.frames.length) last=r.frames[r.frames.length-1]!.activeMidi;} return last; }
function tone(f0:number, ampsDb:number[], noiseDb=-120, sec=0.6, vib=0){ const n=Math.floor(SR*sec); const s=new Float32Array(n); let ph=0; for(let i=0;i<n;i++){ const t=i/SR; const f=f0*Math.pow(2,(vib*Math.sin(2*Math.PI*5.5*t))/1200); ph+=2*Math.PI*f/SR; let v=0; ampsDb.forEach((a,k)=>{v+=Math.pow(10,a/20)*Math.sin((k+1)*ph);}); v+=Math.pow(10,noiseDb/20)*(Math.random()*2-1); s[i]=v*0.3;} return s; }
const midi=(f:number)=>Math.round(69+12*Math.log2(f/440));
const cases: [string, Float32Array, number[]][] = [
  ['baritone A2 chest (H2,H3 > H1)', tone(110,[-20,-8,-9,-16,-12,-22,-26,-30,-34,-38]), [midi(110)]],
  ['baritone D3', tone(146.8,[-14,-6,-12,-10,-18,-24,-28,-32]), [midi(146.8)]],
  ['baritone A2 + vibrato 40c', tone(110,[-20,-8,-9,-16,-12,-22,-26,-30],-120,0.6,40), [midi(110)]],
  ['G3 strong H2 (H2 = H1+12dB)', tone(196,[-16,-4,-10,-12,-14,-20,-24,-28]), [midi(196)]],
  ['noise only -50 dB', tone(110,[-200],-50), []],
  ['breath -35 dB + baritone', tone(110,[-20,-8,-9,-16,-12,-22,-26,-30],-35), [midi(110)]],
];
for(const [name,s,gt] of cases){ const d=run(s); console.log(name.padEnd(34), 'gt',gt,'->',d, d.length===gt.length&&gt.every(g=>d.includes(g))?'OK':'MISMATCH'); }
