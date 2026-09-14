import { runBench, BENCH_CASES } from '../src/audio/choir-bench';
import { StreamingStft } from '../src/audio/streaming-stft';
import { NoteDetector } from '../src/audio/note-detector';
import { DETECTOR_CONFIG } from '../src/config/detector';
const SR=44100;
function mk(margin:number){ return (samples:Float32Array)=>{ const stft=new StreamingStft(8192,2048); const det=new NoteDetector({numBins:4096,sampleRate:SR,minFreqHz:50,maxFreqHz:8000,harmonic:true,harmonicCount:DETECTOR_CONFIG.harmonicCount,salienceThreshold:DETECTOR_CONFIG.salienceThreshold,decayPerFrame:DETECTOR_CONFIG.decayPerFrame,familyMarginDb:margin}); let last=new Set<number>(); for(let o=0;o<samples.length;o+=128){ for(const m of stft.pushSamples(samples.subarray(o,o+128))) last=det.analyze(m);} return last; }; }
const orig=BENCH_CASES.filter(c=>!c.voices.some(v=>v.harmonicDb)); const added=BENCH_CASES.filter(c=>c.voices.some(v=>v.harmonicDb));
for(const m of [0,3,4,5,6,8,10]){ const all=runBench(BENCH_CASES,mk(m)); const o=runBench(orig,mk(m)); const a=runBench(added,mk(m)); const singles=a.cases.filter(c=>c.tags.includes('single')); const ok=singles.filter(c=>c.fp===0&&c.fn===0).length; console.log(`margin ${String(m).padStart(2)}: all F1=${all.overall.f1.toFixed(3)}  original19 F1=${o.overall.f1.toFixed(3)}  added13 F1=${a.overall.f1.toFixed(3)}  singles exact ${ok}/${singles.length}  oct-partner F1=${(a.byTag['oct-partner']?.f1??0).toFixed(2)}`); }
