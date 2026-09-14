import { synthesizeChoirChord } from '../src/audio/choir-synth';
import { SpectrogramDsp } from '../src/audio/dsp-core';
const SR=48000;
const midi=Number(process.argv[2]||45); const strong = process.argv[3]==='strong';
const { samples } = synthesizeChoirChord({ sampleRate: SR, durationSec: 3, seed: 4, voices: [{ section:'B', midi, singers:1, detuneCents:0, vibratoCents:20, harmonicDb: strong? [0,4,2,-3,-6,-10,-14,-18,-22,-26] : undefined }] });
const dsp = new SpectrogramDsp({ fftSize: 2048, hopSize: 512, sampleRate: SR, minFreqHz: 50, maxFreqHz: 8000 }); if (process.argv[4]) dsp.setHarmony({ familyMarginDb: Number(process.argv[4]) });
let t=0; dsp.now=()=>t; let lastKey=''; const lit: Record<string,number>={};
for (let o=0;o<samples.length;o+=128){ t+=128/SR*1000; const r=dsp.pushSamples(samples.subarray(o,o+128));
  for (const h of r.harmony){ const key=h.result.voices.map(v=>v.section+v.midi).join(',');
    for (const v of h.result.voices) lit[v.section]=(lit[v.section]||0)+1;
    if(key!==lastKey){ console.log((t/1000).toFixed(2)+'s', key, 'in:', h.voices.map(v=>v.section+v.midi+'('+v.salience.toFixed(2)+','+v.sectionConf.toFixed(2)+')').join(' ')); lastKey=key; } } }
console.log('frames lit per card', lit);
