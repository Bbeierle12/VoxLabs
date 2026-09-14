import { synthesizeChoirChord } from '../src/audio/choir-synth';
import { StreamingStft } from '../src/audio/streaming-stft';
import { NoteDetector } from '../src/audio/note-detector';
import { DETECTOR_CONFIG } from '../src/config/detector';
const SR=44100;
const { samples } = synthesizeChoirChord({ sampleRate: SR, durationSec: 0.4, seed: 7, voices: [{ section:'B', midi:45, harmonicDb:[-20,-8,-9,-16,-12,-22,-26,-30,-34,-38], singers:1, detuneCents:0 }] });
const stft=new StreamingStft(8192,2048); const d:any=new NoteDetector({numBins:4096,sampleRate:SR,minFreqHz:50,maxFreqHz:8000,harmonic:true,harmonicCount:DETECTOR_CONFIG.harmonicCount,salienceThreshold:DETECTOR_CONFIG.salienceThreshold,decayPerFrame:DETECTOR_CONFIG.decayPerFrame,familyMarginDb:8});
let last:Set<number>=new Set(); let mags:Float32Array|null=null;
for(let o=0;o<samples.length;o+=128){ for(const m of stft.pushSamples(samples.subarray(o,o+128))){ last=d.analyze(m); mags=m; } }
console.log('active', [...last]);
const i45=45-d.midiMin, i57=57-d.midiMin, i64=64-d.midiMin;
console.log('A2 parts dB', d.partialLevels(i45).map((x:any)=>x==null?null:x.toFixed(1)));
console.log('A3 own 1..3', [1,2,3].map(m=>d.partialDb(i57,m)?.toFixed(1)));
const env=d.fitEnvelopeExcluding(d.partialLevels(i45),2); console.log('env(2,4,6)', [2,4,6].map(k=>env(k).toFixed(1)));
console.log('E4 own 1..3', [1,2,3].map(m=>d.partialDb(i64,m)?.toFixed(1)));
const env3=d.fitEnvelopeExcluding(d.partialLevels(i45),3); console.log('env(3,6,9)', [3,6,9].map(k=>env3(k).toFixed(1)));
console.log('conf', [...d.lastConfidences().entries()]);
