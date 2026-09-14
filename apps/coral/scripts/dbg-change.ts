import { synthesizeChoirChord } from '../src/audio/choir-synth';
import { SpectrogramDsp } from '../src/audio/dsp-core';
const SR=48000;
const mk=(m:number[],seed:number)=>synthesizeChoirChord({ sampleRate: SR, durationSec: Number(process.argv[2]||2), seed, voices: (['B','T','A','S'] as const).map((section,i)=>({ section, midi: m[i]!, singers:4, detuneCents:6, vibratoCents:15 })) }).samples;
const a=mk([48,55,64,72],5), b=mk([53,57,60,69],6);
const samples=new Float32Array(a.length+b.length); samples.set(a); samples.set(b,a.length);
const dsp = new SpectrogramDsp({ fftSize: 2048, hopSize: 512, sampleRate: SR, minFreqHz: 50, maxFreqHz: 8000 });
let t=0; dsp.now=()=>t; let lastKey='';
for (let o=0;o<samples.length;o+=128){ t+=128/SR*1000; const r=dsp.pushSamples(samples.subarray(o,o+128));
  for (const h of r.harmony){ const key=(h.result.id? (h.result.id.cluster?'cluster':h.result.id.root+':'+h.result.id.chord.name):'-')+'/'+h.result.voices.map(v=>v.section+v.midi).join(',');
    if(key!==lastKey){ console.log((t/1000).toFixed(2)+'s', key, 'in:', h.voices.map(v=>v.section+v.midi+'('+v.salience.toFixed(2)+','+v.sectionConf.toFixed(2)+')').join(' ')); lastKey=key; } } }
