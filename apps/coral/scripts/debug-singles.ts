import { runBench, BENCH_CASES, formatReport } from '../src/audio/choir-bench';
import { StreamingStft } from '../src/audio/streaming-stft';
import { NoteDetector } from '../src/audio/note-detector';
import { DETECTOR_CONFIG } from '../src/config/detector';
const SR=44100; const margin=+(process.argv[2]??8);
const det=(samples:Float32Array)=>{ const stft=new StreamingStft(8192,2048); const d=new NoteDetector({numBins:4096,sampleRate:SR,minFreqHz:50,maxFreqHz:8000,harmonic:true,harmonicCount:DETECTOR_CONFIG.harmonicCount,salienceThreshold:DETECTOR_CONFIG.salienceThreshold,decayPerFrame:DETECTOR_CONFIG.decayPerFrame,familyMarginDb:margin}); let last=new Set<number>(); for(let o=0;o<samples.length;o+=128){ for(const m of stft.pushSamples(samples.subarray(o,o+128))) last=d.analyze(m);} return last; };
const added=BENCH_CASES.filter(c=>c.voices.some(v=>v.harmonicDb));
console.log(formatReport(runBench(added,det)).split('\n').filter(l=>l.includes('->')).join('\n'));
