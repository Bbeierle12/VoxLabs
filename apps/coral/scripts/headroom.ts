import { runBench, BENCH_CASES } from '../src/audio/choir-bench';
import { StreamingStft } from '../src/audio/streaming-stft';
import { NoteDetector } from '../src/audio/note-detector';
import { DETECTOR_CONFIG } from '../src/config/detector';
const SR=44100;
function mk(headroom:number){ return (samples:Float32Array)=>{ const stft=new StreamingStft(8192,2048); const det=new NoteDetector({numBins:4096,sampleRate:SR,minFreqHz:50,maxFreqHz:8000,harmonic:true,harmonicCount:DETECTOR_CONFIG.harmonicCount,salienceThreshold:DETECTOR_CONFIG.salienceThreshold,decayPerFrame:DETECTOR_CONFIG.decayPerFrame,cancelHeadroom:headroom}); let last=new Set<number>(); for(let o=0;o<samples.length;o+=128){ for(const m of stft.pushSamples(samples.subarray(o,o+128))) last=det.analyze(m);} return last; }; }
function tone(f0:number, ampsDb:number[], sec=0.6){ const n=Math.floor(SR*sec); const s=new Float32Array(n); let ph=0; for(let i=0;i<n;i++){ ph+=2*Math.PI*f0/SR; let v=0; ampsDb.forEach((a,k)=>{v+=Math.pow(10,a/20)*Math.sin((k+1)*ph);}); s[i]=v*0.3;} return s; }
const bari=[tone(110,[-20,-8,-9,-16,-12,-22,-26,-30,-34,-38]), tone(146.8,[-14,-6,-12,-10,-18,-24,-28,-32]), tone(196,[-16,-4,-10,-12,-14,-20,-24,-28])];
for(const h of [1.0,1.5,2.0,2.5,3.0]){ const det=mk(h); const rep=runBench(BENCH_CASES,det); const single=bari.map(s=>[...det(s)].join(',')); console.log(`headroom ${h}: bench F1=${rep.overall.f1.toFixed(3)} P=${rep.overall.precision.toFixed(3)} R=${rep.overall.recall.toFixed(3)} | single voices -> ${single.join(' | ')}`); }
